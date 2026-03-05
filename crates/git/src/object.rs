use gengo::{Builder as LanguageBuilder, Git as LanguageGit};
use git2::{ObjectType, Repository, Tree};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ObjectError {
  #[error("invalid path: {0}")]
  InvalidPath(String),
  #[error("failed to open repository: {0}")]
  Open(String),
  #[error("failed to resolve branch: {0}")]
  ResolveBranch(String),
  #[error("failed to resolve commit: {0}")]
  ResolveCommit(String),
  #[error("failed to resolve tree: {0}")]
  ResolveTree(String),
  #[error("failed to resolve blob: {0}")]
  ResolveBlob(String),
  #[error("failed to analyze repository languages: {0}")]
  Language(String),
  #[error("{0} not found")]
  NotFound(String),
  #[error("{0} is not a directory")]
  NotDirectory(String),
  #[error("{0} is not a file")]
  NotBlob(String),
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
  pub name: String,
  pub path: String,
  pub kind: String,
  pub oid: String,
  pub size: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct BlobObject {
  pub path: String,
  pub content: Vec<u8>,
  pub size: usize,
}

#[derive(Debug, Clone)]
pub struct CommitObject {
  pub commit_sha: String,
  pub message: String,
  pub author: String,
  pub authored_at: i64,
}

#[derive(Debug, Clone)]
pub struct RepositoryLanguageStat {
  pub language: String,
  pub bytes: u64,
}

pub fn list_tree_entries(
  repo_path: &Path,
  branch: &str,
  dir_path: Option<&str>,
) -> Result<Vec<TreeEntry>, ObjectError> {
  let repo = Repository::open_bare(repo_path).map_err(|err| ObjectError::Open(err.to_string()))?;
  let segments = normalize_path(dir_path)?;
  let tree = resolve_tree_by_branch_and_segments(&repo, branch, &segments)?;
  let prefix = segments.join("/");

  let mut items: Vec<TreeEntry> = tree
    .iter()
    .filter_map(|entry| {
      let name = String::from_utf8_lossy(entry.name_bytes()).to_string();
      if name.is_empty() {
        return None;
      }

      let kind = object_kind(entry.kind());
      let path = if prefix.is_empty() {
        name.clone()
      } else {
        format!("{prefix}/{name}")
      };

      let size = match entry.kind() {
        Some(ObjectType::Blob) => repo
          .find_blob(entry.id())
          .ok()
          .map(|blob| blob.content().len()),
        _ => None,
      };

      Some(TreeEntry {
        name,
        path,
        kind,
        oid: entry.id().to_string(),
        size,
      })
    })
    .collect();

  items.sort_by(|left, right| {
    let left_rank = if left.kind == "tree" { 0 } else { 1 };
    let right_rank = if right.kind == "tree" { 0 } else { 1 };
    left_rank
      .cmp(&right_rank)
      .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
  });

  Ok(items)
}

pub fn read_blob(
  repo_path: &Path,
  branch: &str,
  file_path: &str,
) -> Result<BlobObject, ObjectError> {
  let repo = Repository::open_bare(repo_path).map_err(|err| ObjectError::Open(err.to_string()))?;
  let full_segments = normalize_path(Some(file_path))?;
  if full_segments.is_empty() {
    return Err(ObjectError::InvalidPath(
      "file path cannot be empty".to_string(),
    ));
  }

  let file_name = full_segments.last().cloned().ok_or_else(|| {
    ObjectError::InvalidPath("file path cannot be empty".to_string())
  })?;
  let dir_segments = full_segments
    .iter()
    .take(full_segments.len().saturating_sub(1))
    .cloned()
    .collect::<Vec<_>>();
  let tree = resolve_tree_by_branch_and_segments(&repo, branch, &dir_segments)?;

  let entry = tree
    .get_name(file_name.as_str())
    .ok_or_else(|| ObjectError::NotFound(full_segments.join("/")))?;
  if entry.kind() != Some(ObjectType::Blob) {
    return Err(ObjectError::NotBlob(full_segments.join("/")));
  }

  let blob = repo
    .find_blob(entry.id())
    .map_err(|err| ObjectError::ResolveBlob(err.to_string()))?;
  let content = blob.content().to_vec();
  let size = content.len();

  Ok(BlobObject {
    path: full_segments.join("/"),
    content,
    size,
  })
}

pub fn read_root_readme(repo_path: &Path, branch: &str) -> Result<Option<BlobObject>, ObjectError> {
  let repo = Repository::open_bare(repo_path).map_err(|err| ObjectError::Open(err.to_string()))?;
  let tree = resolve_tree_by_branch_and_segments(&repo, branch, &[])?;

  let mut readme_entries: Vec<(String, git2::Oid)> = tree
    .iter()
    .filter_map(|entry| {
      if entry.kind() != Some(ObjectType::Blob) {
        return None;
      }
      let name = String::from_utf8_lossy(entry.name_bytes()).to_string();
      if name.is_empty() || !name.to_ascii_lowercase().starts_with("readme") {
        return None;
      }
      Some((name, entry.id()))
    })
    .collect();

  if readme_entries.is_empty() {
    return Ok(None);
  }

  readme_entries.sort_by(|left, right| {
    readme_rank(left.0.as_str())
      .cmp(&readme_rank(right.0.as_str()))
      .then_with(|| left.0.to_lowercase().cmp(&right.0.to_lowercase()))
  });

  let (path, oid) = readme_entries.remove(0);
  let blob = repo
    .find_blob(oid)
    .map_err(|err| ObjectError::ResolveBlob(err.to_string()))?;
  let content = blob.content().to_vec();
  let size = content.len();

  Ok(Some(BlobObject {
    path,
    content,
    size,
  }))
}

pub fn list_commits(
  repo_path: &Path,
  branch: &str,
  limit: usize,
) -> Result<Vec<CommitObject>, ObjectError> {
  let repo = Repository::open_bare(repo_path).map_err(|err| ObjectError::Open(err.to_string()))?;
  let resolved_limit = if limit == 0 { 50 } else { limit };
  let head_commit = resolve_branch_commit(&repo, branch)?;

  let mut revwalk = repo
    .revwalk()
    .map_err(|err| ObjectError::ResolveCommit(err.to_string()))?;
  revwalk
    .push(head_commit.id())
    .map_err(|err| ObjectError::ResolveCommit(err.to_string()))?;
  revwalk
    .set_sorting(git2::Sort::TIME | git2::Sort::TOPOLOGICAL)
    .map_err(|err| ObjectError::ResolveCommit(err.to_string()))?;

  let mut commits = Vec::new();
  for oid in revwalk.take(resolved_limit) {
    let oid = oid.map_err(|err| ObjectError::ResolveCommit(err.to_string()))?;
    let commit = repo
      .find_commit(oid)
      .map_err(|err| ObjectError::ResolveCommit(err.to_string()))?;
    let summary = commit
      .summary()
      .map(str::trim)
      .filter(|value| !value.is_empty())
      .map(str::to_string)
      .or_else(|| {
        commit
          .message()
          .map(str::trim)
          .filter(|value| !value.is_empty())
          .map(str::to_string)
      })
      .unwrap_or_else(|| "Commit".to_string());

    let signature = commit.author();
    let author = match (signature.name(), signature.email()) {
      (Some(name), Some(email)) if !name.trim().is_empty() => format!("{name} <{email}>"),
      (Some(name), _) if !name.trim().is_empty() => name.to_string(),
      (_, Some(email)) => email.to_string(),
      _ => "unknown".to_string(),
    };

    commits.push(CommitObject {
      commit_sha: oid.to_string(),
      message: summary,
      author,
      authored_at: commit.time().seconds(),
    });
  }

  Ok(commits)
}

pub fn summarize_languages(
  repo_path: &Path,
  revision: &str,
) -> Result<Vec<RepositoryLanguageStat>, ObjectError> {
  let normalized_revision = revision.trim();
  if normalized_revision.is_empty() {
    return Err(ObjectError::InvalidPath(
      "revision cannot be empty".to_string(),
    ));
  }

  let git = LanguageGit::new(repo_path, normalized_revision)
    .map_err(|err| ObjectError::Language(format!("failed to open git source: {err}")))?;
  let gengo = LanguageBuilder::new(git)
    .build()
    .map_err(|err| ObjectError::Language(format!("failed to build analyzer: {err}")))?;
  let analysis = gengo
    .analyze()
    .map_err(|err| ObjectError::Language(format!("failed to analyze repository: {err}")))?;
  let summary = analysis.summary();

  let mut stats = summary
    .iter()
    .map(|(language, size)| RepositoryLanguageStat {
      language: format!("{language:?}"),
      bytes: *size as u64,
    })
    .collect::<Vec<_>>();

  stats.sort_by(|left, right| {
    right
      .bytes
      .cmp(&left.bytes)
      .then_with(|| left.language.cmp(&right.language))
  });

  Ok(stats)
}

fn resolve_tree_by_branch_and_segments<'a>(
  repo: &'a Repository,
  branch: &str,
  segments: &[String],
) -> Result<Tree<'a>, ObjectError> {
  let mut tree = resolve_root_tree(repo, branch)?;

  for (index, segment) in segments.iter().enumerate() {
    let current_path = segments[..=index].join("/");
    let (entry_id, entry_kind) = {
      let entry = tree
        .get_name(segment.as_str())
        .ok_or_else(|| ObjectError::NotFound(current_path.clone()))?;
      (entry.id(), entry.kind())
    };

    if entry_kind != Some(ObjectType::Tree) {
      return Err(ObjectError::NotDirectory(current_path));
    }
    tree = repo
      .find_tree(entry_id)
      .map_err(|err| ObjectError::ResolveTree(err.to_string()))?;
  }

  Ok(tree)
}

fn resolve_root_tree<'a>(repo: &'a Repository, branch: &str) -> Result<Tree<'a>, ObjectError> {
  let commit = resolve_branch_commit(repo, branch)?;
  commit
    .tree()
    .map_err(|err| ObjectError::ResolveTree(err.to_string()))
}

fn resolve_branch_commit<'a>(
  repo: &'a Repository,
  branch: &str,
) -> Result<git2::Commit<'a>, ObjectError> {
  let normalized_branch = branch.trim();
  if normalized_branch.is_empty() {
    return Err(ObjectError::InvalidPath("branch cannot be empty".to_string()));
  }

  let head_ref = format!("refs/heads/{normalized_branch}");
  let commit = match repo.refname_to_id(head_ref.as_str()) {
    Ok(oid) => repo
      .find_commit(oid)
      .map_err(|err| ObjectError::ResolveCommit(err.to_string()))?,
    Err(_) => {
      let object = repo
        .revparse_single(normalized_branch)
        .map_err(|err| ObjectError::ResolveBranch(err.to_string()))?;
      object
        .peel_to_commit()
        .map_err(|err| ObjectError::ResolveCommit(err.to_string()))?
    }
  };

  Ok(commit)
}

fn normalize_path(path: Option<&str>) -> Result<Vec<String>, ObjectError> {
  let Some(path) = path.map(str::trim) else {
    return Ok(vec![]);
  };
  if path.is_empty() {
    return Ok(vec![]);
  }

  let normalized = path.replace('\\', "/");
  let normalized = normalized.trim_matches('/');
  if normalized.is_empty() {
    return Ok(vec![]);
  }

  let mut segments = Vec::new();
  for segment in normalized.split('/') {
    if segment.is_empty() || segment == "." || segment == ".." {
      return Err(ObjectError::InvalidPath(path.to_string()));
    }
    if segment.contains('\0') {
      return Err(ObjectError::InvalidPath(path.to_string()));
    }
    segments.push(segment.to_string());
  }

  Ok(segments)
}

fn object_kind(kind: Option<ObjectType>) -> String {
  match kind {
    Some(ObjectType::Tree) => "tree".to_string(),
    Some(ObjectType::Blob) => "blob".to_string(),
    Some(ObjectType::Commit) => "commit".to_string(),
    Some(ObjectType::Tag) => "tag".to_string(),
    _ => "unknown".to_string(),
  }
}

fn readme_rank(name: &str) -> usize {
  let lower = name.to_ascii_lowercase();
  match lower.as_str() {
    "readme.md" => 0,
    "readme.markdown" => 1,
    "readme.rst" => 2,
    "readme.txt" => 3,
    "readme" => 4,
    _ => 10,
  }
}
