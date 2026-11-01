use crate::{commit, timestamp, validate_path, GitService};
use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use git2::{ObjectType, Repository, Sort};
use serde_json::{json, Value};
use std::path::Path;
impl GitService {
  pub async fn get_blob(
    &self,
    repo: &str,
    revision: &str,
    default: &str,
    path: &str,
  ) -> Result<Value> {
    self
      .get_blob_limited(repo, revision, default, path, usize::MAX)
      .await
  }
  pub async fn get_blob_limited(
    &self,
    repo: &str,
    revision: &str,
    default: &str,
    path: &str,
    maximum: usize,
  ) -> Result<Value> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::GetBlob {
            revision: revision.into(),
            default: default.into(),
            path: path.into(),
            max_bytes: maximum,
          },
        )
        .await;
    }
    validate_path(path, false)?;
    let (revision, default, path) = (revision.to_owned(), default.to_owned(), path.to_owned());
    self
      .read(repo, move |r| {
        let tree = commit(&r, &revision, &default)?.tree()?;
        let entry = tree
          .get_path(Path::new(&path))
          .context("git path not found")?;
        if r.odb()?.read_header(entry.id())?.0 > maximum {
          bail!("Git blob exceeds permitted size");
        }
        blob(&r, entry.id(), &path)
      })
      .await
  }

  pub async fn get_readme(&self, repo: &str, revision: &str, default: &str) -> Result<Value> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::GetReadme {
            revision: revision.into(),
            default: default.into(),
          },
        )
        .await;
    }
    let (revision, default) = (revision.to_owned(), default.to_owned());
    self
      .read(repo, move |r| {
        let tree = commit(&r, &revision, &default)?.tree()?;
        for entry in tree.iter() {
          let name = entry.name().unwrap_or_default();
          if [
            "readme",
            "readme.md",
            "readme.txt",
            "readme.rst",
            "readme.adoc",
          ]
          .contains(&name.to_lowercase().as_str())
            && entry.kind() == Some(ObjectType::Blob)
          {
            return blob(&r, entry.id(), name);
          }
        }
        bail!("git readme not found")
      })
      .await
  }

  pub async fn list_commits(
    &self,
    repo: &str,
    revision: &str,
    default: &str,
    limit: usize,
  ) -> Result<Value> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::ListCommits {
            revision: revision.into(),
            default: default.into(),
            limit,
          },
        )
        .await;
    }
    let (revision, default) = (revision.to_owned(), default.to_owned());
    self
      .read(repo, move |r| {
        if crate::is_empty(&r)? {
          return Ok(json!([]));
        }
        let mut walk = r.revwalk()?;
        walk.push(commit(&r, &revision, &default)?.id())?;
        walk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;
        let mut commits = Vec::new();
        for id in walk.take(if limit == 0 { 20 } else { limit.min(100) }) {
          let c = r.find_commit(id?)?;
          let hash = c.id().to_string();
          commits.push(json!({
                      "hash":hash,
                      "short_hash":&hash[..8],
                      "author_name":c.author().name().unwrap_or_default(),
                      "author_email":c.author().email().unwrap_or_default(),
                      "message":c.message().unwrap_or_default().trim(),
                      "committed_at":timestamp(c.author().when().seconds())
                  }
          ));
        }
        Ok(json!(commits))
      })
      .await
  }
}

fn blob(repo: &Repository, id: git2::Oid, path: &str) -> Result<Value> {
  let blob = repo.find_blob(id)?;
  let text = std::str::from_utf8(blob.content())
    .ok()
    .filter(|s| !s.contains('\0'));
  Ok(json!({
          "name":path.rsplit('/').next().unwrap_or(path),
          "path":path,
          "size":blob.size(),
          "encoding":if text.is_some(){
              "utf-8"
          }
          else{
              "base64"
          }
          ,
          "content":text.map(str::to_owned).unwrap_or_else(||STANDARD.encode(blob.content()))
      }
  ))
}
