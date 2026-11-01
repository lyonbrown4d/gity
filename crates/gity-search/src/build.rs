use crate::*;
use git2::Repository;
use tantivy::{Index, TantivyDocument, doc};
impl SearchIndex {
  pub async fn refresh(
    &self,
    project_id: i64,
    repo: &str,
    default_branch: &str,
  ) -> Result<IndexStatus> {
    let path = self.git.repository_path(repo)?;
    let branch = default_branch.to_owned();
    let service = self.clone();
    tokio::task::spawn_blocking(move || service.rebuild(project_id, &path, &branch)).await?
  }
  fn rebuild(&self, project_id: i64, repository: &Path, branch: &str) -> Result<IndexStatus> {
    let _lock = self
      .lock(project_id, true)?
      .context("search index refresh already running")?;
    let repo = Repository::open_bare(repository)?;
    if repo.references()?.next().transpose()?.is_none() {
      let path = self.path(project_id)?;
      if path.exists() {
        std::fs::remove_dir_all(path)?;
      }
      return Ok(IndexStatus {
        revision: String::new(),
        indexed_files: 0,
        total_bytes: 0,
        truncated: false,
        unchanged: false,
      });
    }
    let object = repo
      .revparse_single(&format!("refs/heads/{branch}"))
      .or_else(|_| repo.revparse_single("HEAD"))?;
    let commit = object.peel_to_commit()?;
    let revision = commit.id().to_string();
    let path = self.path(project_id)?;
    std::fs::create_dir_all(&path)?;
    let index = if path.join("meta.json").exists() {
      open(&path)?
    } else {
      let (schema, _) = schema::schema();
      let index = Index::create_in_dir(&path, schema)?;
      schema::register(&index)?;
      index
    };
    if let Some(metadata) = metadata(&index)?
      && same_limits(&metadata, self.limits)
      && metadata.status.revision == revision
    {
      return Ok(IndexStatus {
        unchanged: true,
        ..metadata.status
      });
    }
    let fields = schema::fields(&index)?;
    let mut writer = index.writer_with_num_threads::<TantivyDocument>(1, 50_000_000)?;
    writer.delete_all_documents()?;
    let mut status = IndexStatus {
      revision,
      indexed_files: 0,
      total_bytes: 0,
      truncated: false,
      unchanged: false,
    };
    let mut failure = None;
    gity_git::walk_blobs(
      &repo,
      &commit.tree()?,
      self.limits.max_file_size,
      &mut |path, blob| {
        let Ok(content) = std::str::from_utf8(blob.content()) else {
          return true;
        };
        if content.contains('\0') {
          return true;
        }
        if status.indexed_files >= self.limits.max_files
          || status.total_bytes.saturating_add(content.len()) > self.limits.max_total_bytes
        {
          status.truncated = true;
          return false;
        }
        if let Err(error) =
                    writer.add_document(doc!(fields.path=>path,fields.content=>content,fields.candidate=>content.replace('ſ',"s")))
                {
                    failure = Some(error);
                    return false;
                }
        status.indexed_files += 1;
        status.total_bytes += content.len();
        true
      },
    )?;
    if let Some(error) = failure {
      writer.rollback()?;
      return Err(error.into());
    }
    let payload = serde_json::to_string(&Metadata {
      version: 1,
      status: status.clone(),
      max_files: self.limits.max_files,
      max_file_size: self.limits.max_file_size,
      max_total_bytes: self.limits.max_total_bytes,
    })?;
    let mut prepared = writer.prepare_commit()?;
    prepared.set_payload(&payload);
    prepared.commit()?;
    writer.wait_merging_threads()?;
    tracing::info!(project_id,revision=%status.revision,indexed_files=status.indexed_files,total_bytes=status.total_bytes,truncated=status.truncated,"search index refreshed");
    Ok(status)
  }
}
