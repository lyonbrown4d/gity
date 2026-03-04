use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
  #[error("invalid default branch: {0}")]
  InvalidDefaultBranch(String),
  #[error("repository already exists at {0}")]
  AlreadyExists(PathBuf),
  #[error("failed to init bare repository: {0}")]
  Init(String),
  #[error("failed to open repository: {0}")]
  Open(String),
  #[error("failed to read repository refs: {0}")]
  Refs(String),
  #[error("failed to write repository HEAD: {0}")]
  HeadWrite(String),
}

pub fn init_bare_repository(repo_path: &Path, default_branch: &str) -> Result<(), StorageError> {
  let normalized_branch = default_branch.trim();
  if !is_valid_default_branch(normalized_branch) {
    return Err(StorageError::InvalidDefaultBranch(
      "default branch cannot be empty or contain unsupported characters".to_string(),
    ));
  }

  if repo_path.exists() {
    return Err(StorageError::AlreadyExists(repo_path.to_path_buf()));
  }

  git2::Repository::init_bare(repo_path).map_err(|err| StorageError::Init(err.to_string()))?;
  let head = format!("ref: refs/heads/{normalized_branch}\n");
  std::fs::write(repo_path.join("HEAD"), head)
    .map_err(|err| StorageError::HeadWrite(err.to_string()))?;

  Ok(())
}

pub fn list_head_refs(repo_path: &Path) -> Result<Vec<(String, String)>, StorageError> {
  let repo =
    git2::Repository::open_bare(repo_path).map_err(|err| StorageError::Open(err.to_string()))?;
  let references = repo
    .references_glob("refs/heads/*")
    .map_err(|err| StorageError::Refs(err.to_string()))?;

  let mut refs = Vec::new();
  for item in references {
    let reference = item.map_err(|err| StorageError::Refs(err.to_string()))?;
    let Some(id) = reference.target() else {
      continue;
    };
    let Some(full_name) = reference.name() else {
      continue;
    };
    let Some(branch_name) = full_name.strip_prefix("refs/heads/") else {
      continue;
    };
    refs.push((branch_name.to_string(), id.to_string()));
  }
  refs.sort_by(|left, right| left.0.cmp(&right.0));
  Ok(refs)
}

fn is_valid_default_branch(value: &str) -> bool {
  !value.is_empty()
    && !value.contains('\\')
    && !value.contains(' ')
    && !value.contains("..")
    && !value.starts_with('/')
    && !value.ends_with('/')
}
