use super::{signature, CreateFileCommitInput};
use anyhow::{bail, Context, Result};
use git2::{Index, IndexEntry, IndexTime, ObjectType, Oid, Repository};
use serde_json::{json, Value};
use std::path::Path;

pub(super) fn create(
  repo: &Repository,
  reference: &str,
  input: &CreateFileCommitInput,
) -> Result<Value> {
  let old = match repo.find_reference(reference) {
    Ok(reference) => Some(
      reference
        .target()
        .context("branch is not a direct reference")?,
    ),
    Err(error) if error.code() == git2::ErrorCode::NotFound => None,
    Err(error) => return Err(error.into()),
  };
  if input.update_existing && input.expected_head_sha.as_deref().is_none_or(str::is_empty) {
    bail!("updating a file requires expected_head_sha");
  }
  if let Some(expected) = &input.expected_head_sha {
    let expected = if expected.is_empty() {
      None
    } else {
      Some(Oid::from_str(expected).context("invalid expected_head_sha")?)
    };
    if expected != old {
      bail!("branch head changed; reload before committing");
    }
  }
  let parent = match old {
    Some(id) => Some(repo.find_commit(id)?),
    None => repo.head().ok().and_then(|head| head.peel_to_commit().ok()),
  };
  let mut index = Index::new()?;
  let mut mode = 0o100644;
  if let Some(parent) = &parent {
    let tree = parent.tree()?;
    index.read_tree(&tree)?;
    let existing = tree.get_path(Path::new(&input.file_path)).ok();
    match existing {
      Some(entry) if input.update_existing => {
        if entry.kind() != Some(ObjectType::Blob) {
          bail!("file update requires an existing blob");
        }
        mode = entry.filemode() as u32;
      }
      Some(_) => bail!("git file already exists"),
      None if input.update_existing => bail!("git file not found"),
      None => {}
    }
    let mut ancestor = Path::new(&input.file_path).parent();
    while let Some(path) = ancestor.filter(|path| !path.as_os_str().is_empty()) {
      if let Ok(entry) = tree.get_path(path) {
        if entry.kind() != Some(ObjectType::Tree) {
          bail!("file parent is not a directory");
        }
      }
      ancestor = path.parent();
    }
  } else if input.update_existing {
    bail!("git file not found");
  }
  let id = repo.blob(input.content.as_bytes())?;
  index.add(&IndexEntry {
    ctime: IndexTime::new(0, 0),
    mtime: IndexTime::new(0, 0),
    dev: 0,
    ino: 0,
    mode,
    uid: 0,
    gid: 0,
    file_size: input
      .content
      .len()
      .try_into()
      .context("file is too large")?,
    id,
    flags: 0,
    flags_extended: 0,
    path: input.file_path.as_bytes().to_vec(),
  })?;
  let tree = repo.find_tree(index.write_tree_to(repo)?)?;
  let signature = signature(&input.author_name, &input.author_email)?;
  let parents = parent.iter().collect::<Vec<_>>();
  let id = repo.commit(
    None,
    &signature,
    &signature,
    &input.message,
    &tree,
    &parents,
  )?;
  update_tip(repo, reference, id, old)?;
  Ok(json!({"hash":id.to_string()}))
}

fn update_tip(repo: &Repository, reference: &str, id: Oid, old: Option<Oid>) -> Result<()> {
  let result = if let Some(old) = old {
    repo.reference_matching(reference, id, true, old, "Gity commit")
  } else {
    repo.reference(reference, id, false, "Gity initial commit")
  };
  result.map_err(|error| match error.code() {
    git2::ErrorCode::Modified
    | git2::ErrorCode::Exists
    | git2::ErrorCode::Locked
    | git2::ErrorCode::NotFound => {
      anyhow::anyhow!("branch head changed; reload before committing")
    }
    _ => error.into(),
  })?;
  Ok(())
}
