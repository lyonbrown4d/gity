use crate::{commit, timestamp, validate_path, GitService};
use anyhow::Result;
use git2::{BranchType, ObjectType};
use serde_json::{json, Value};
use std::path::Path;

impl GitService {
  pub async fn file_exists(
    &self,
    repo: &str,
    revision: &str,
    default: &str,
    path: &str,
  ) -> Result<bool> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::FileExists {
            revision: revision.into(),
            default: default.into(),
            path: path.into(),
          },
        )
        .await;
    }
    validate_path(path, false)?;
    let (revision, default, path) = (revision.to_owned(), default.to_owned(), path.to_owned());
    let value = self
      .read(repo, move |r| {
        if crate::is_empty(&r)? {
          return Ok(json!(false));
        }
        let tree = commit(&r, &revision, &default)?.tree()?;
        match tree.get_path(Path::new(&path)) {
          Ok(entry) => Ok(json!(entry.kind() == Some(ObjectType::Blob))),
          Err(error) if error.code() == git2::ErrorCode::NotFound => Ok(json!(false)),
          Err(error) => Err(error.into()),
        }
      })
      .await?;
    Ok(value.as_bool().unwrap_or(false))
  }

  pub async fn list_branches(&self, repo: &str, default: &str) -> Result<Value> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::ListBranches {
            default: default.into(),
          },
        )
        .await;
    }
    let default = default.to_owned();
    self.read(repo, move |r| {
                let mut branches = Vec::new();
                for branch in r.branches(Some(BranchType::Local))? {
                    let (branch, _) = branch?;
                    let name = branch.name()?.unwrap_or_default();
                    branches.push(json!({"name":name,"hash":branch.get().target().map(|id| id.to_string()).unwrap_or_default(),"is_default":name==default}));
                }
                branches.sort_by(|a,b| b["is_default"].as_bool().cmp(&a["is_default"].as_bool()).then_with(|| a["name"].as_str().cmp(&b["name"].as_str())));
                Ok(json!(branches))
        }).await
  }

  pub async fn list_tags(&self, repo: &str) -> Result<Value> {
    if self.remote.is_some() {
      return self
        .execute_remote(repo, gity_git_wire::GitOperation::ListTags)
        .await;
    }
    self
      .read(repo, |r| {
        let mut tags = Vec::new();
        for name in r.tag_names(None)?.iter() {
          let Some(name) = name? else { continue };
          let object = r.revparse_single(&format!("refs/tags/{name}"))?;
          let annotated = object.kind() == Some(ObjectType::Tag);
          let mut tag = json!({
                  "name":name,
                  "target_sha":object.id().to_string(),
                  "object_sha":object.id().to_string(),
                  "message":"",
                  "created_at":"",
                  "annotated":annotated,
                  "object_type":"commit"
              }
          );
          if let Some(value) = object.as_tag() {
            tag["target_sha"] = json!(value.target_id().to_string());
            tag["message"] = json!(value.message()?.unwrap_or_default().trim());
            tag["object_type"] = json!(value.target_type().map(|k| k.str()).unwrap_or("unknown"));
            tag["created_at"] = json!(value
              .tagger()
              .map(|s| timestamp(s.when().seconds()))
              .unwrap_or_default());
          } else if let Ok(value) = object.peel_to_commit() {
            tag["created_at"] = json!(timestamp(value.time().seconds()));
          }
          tags.push(tag);
        }
        tags.sort_by(|a, b| {
          b["created_at"]
            .as_str()
            .cmp(&a["created_at"].as_str())
            .then_with(|| a["name"].as_str().cmp(&b["name"].as_str()))
        });
        Ok(json!(tags))
      })
      .await
  }

  pub async fn list_tree(
    &self,
    repo: &str,
    revision: &str,
    default: &str,
    path: &str,
  ) -> Result<Value> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::ListTree {
            revision: revision.into(),
            default: default.into(),
            path: path.into(),
          },
        )
        .await;
    }
    validate_path(path, true)?;
    let (revision, default, path) = (revision.to_owned(), default.to_owned(), path.to_owned());
    self
      .read(repo, move |r| {
        if crate::is_empty(&r)? {
          return Ok(json!([]));
        }
        let root = commit(&r, &revision, &default)?.tree()?;
        let tree = if path.is_empty() {
          root
        } else {
          root
            .get_path(Path::new(&path))?
            .to_object(&r)?
            .peel_to_tree()?
        };
        let mut entries = Vec::new();
        for entry in tree.iter() {
          let name = entry.name().unwrap_or_default();
          let kind = match entry.kind() {
            Some(ObjectType::Tree) => "tree",
            Some(ObjectType::Commit) => "commit",
            _ => "blob",
          };
          let size = if kind == "blob" {
            r.find_blob(entry.id()).map(|b| b.size()).unwrap_or(0)
          } else {
            0
          };
          entries.push(json!({
                      "name":name,
                      "path":if path.is_empty(){
                          name.to_owned()
                      }
                      else{
                          format!("{path}/{name}")
                      }
                      ,
                      "type":kind,
                      "mode":format!("{:06o}",
                          entry.filemode()),
                      "size":size
                  }
          ));
        }
        entries.sort_by(|a, b| {
          (b["type"] == "tree")
            .cmp(&(a["type"] == "tree"))
            .then_with(|| a["name"].as_str().cmp(&b["name"].as_str()))
        });
        Ok(json!(entries))
      })
      .await
  }
}
