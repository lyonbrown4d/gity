use git2::{ObjectType, Repository, Tree};
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
  #[error("invalid initial file path: {0}")]
  InvalidInitialFilePath(String),
  #[error("failed to open bare repository: {0}")]
  OpenBare(String),
  #[error("failed to write git blob: {0}")]
  BlobWrite(String),
  #[error("failed to write git tree: {0}")]
  TreeWrite(String),
  #[error("failed to resolve git tree: {0}")]
  TreeLookup(String),
  #[error("failed to create git signature: {0}")]
  Signature(String),
  #[error("failed to create initial commit: {0}")]
  CommitWrite(String),
  #[error("invalid file path: {0}")]
  InvalidFilePath(String),
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

pub fn seed_initial_commit(
  repo_path: &Path,
  default_branch: &str,
  files: &[(String, String)],
  commit_message: &str,
) -> Result<Option<String>, StorageError> {
  if files.is_empty() {
    return Ok(None);
  }

  let normalized_branch = default_branch.trim();
  if !is_valid_default_branch(normalized_branch) {
    return Err(StorageError::InvalidDefaultBranch(
      "default branch cannot be empty or contain unsupported characters".to_string(),
    ));
  }

  let message = commit_message.trim();
  if message.is_empty() {
    return Err(StorageError::CommitWrite(
      "commit message cannot be empty".to_string(),
    ));
  }

  let repo = git2::Repository::open_bare(repo_path)
    .map_err(|err| StorageError::OpenBare(err.to_string()))?;
  let mut tree = repo
    .treebuilder(None)
    .map_err(|err| StorageError::TreeWrite(err.to_string()))?;

  for (path, content) in files {
    if !is_valid_root_file_path(path.as_str()) {
      return Err(StorageError::InvalidInitialFilePath(path.clone()));
    }
    let blob_id = repo
      .blob(content.as_bytes())
      .map_err(|err| StorageError::BlobWrite(err.to_string()))?;
    tree
      .insert(path.as_str(), blob_id, 0o100644)
      .map_err(|err| StorageError::TreeWrite(err.to_string()))?;
  }

  let tree_id = tree
    .write()
    .map_err(|err| StorageError::TreeWrite(err.to_string()))?;
  let tree = repo
    .find_tree(tree_id)
    .map_err(|err| StorageError::TreeLookup(err.to_string()))?;

  let signature = git2::Signature::now("gity", "noreply@gity.local")
    .map_err(|err| StorageError::Signature(err.to_string()))?;
  let ref_name = format!("refs/heads/{normalized_branch}");
  let commit_id = repo
    .commit(
      Some(ref_name.as_str()),
      &signature,
      &signature,
      message,
      &tree,
      &[],
    )
    .map_err(|err| StorageError::CommitWrite(err.to_string()))?;

  Ok(Some(commit_id.to_string()))
}

pub fn commit_file(
  repo_path: &Path,
  branch: &str,
  file_path: &str,
  content: &str,
  commit_message: &str,
  author_name: &str,
  author_email: &str,
) -> Result<String, StorageError> {
  let normalized_branch = branch.trim();
  if !is_valid_default_branch(normalized_branch) {
    return Err(StorageError::InvalidDefaultBranch(
      "branch cannot be empty or contain unsupported characters".to_string(),
    ));
  }

  let normalized_path = normalize_repo_file_path(file_path)?;
  let message = commit_message.trim();
  if message.is_empty() {
    return Err(StorageError::CommitWrite(
      "commit message cannot be empty".to_string(),
    ));
  }

  let repo = Repository::open_bare(repo_path).map_err(|err| StorageError::Open(err.to_string()))?;
  let ref_name = format!("refs/heads/{normalized_branch}");
  let parent_oid = repo
    .refname_to_id(ref_name.as_str())
    .map_err(|err| StorageError::Refs(err.to_string()))?;
  let parent_commit = repo
    .find_commit(parent_oid)
    .map_err(|err| StorageError::Refs(err.to_string()))?;
  let parent_tree = parent_commit
    .tree()
    .map_err(|err| StorageError::TreeLookup(err.to_string()))?;

  let blob_id = repo
    .blob(content.as_bytes())
    .map_err(|err| StorageError::BlobWrite(err.to_string()))?;
  let tree_id = upsert_blob_into_tree(&repo, Some(&parent_tree), &normalized_path, blob_id)?;
  let tree = repo
    .find_tree(tree_id)
    .map_err(|err| StorageError::TreeLookup(err.to_string()))?;

  let signature = git2::Signature::now(
    normalize_author_name(author_name).as_str(),
    normalize_author_email(author_email).as_str(),
  )
  .map_err(|err| StorageError::Signature(err.to_string()))?;

  let commit_id = repo
    .commit(
      Some(ref_name.as_str()),
      &signature,
      &signature,
      message,
      &tree,
      &[&parent_commit],
    )
    .map_err(|err| StorageError::CommitWrite(err.to_string()))?;

  Ok(commit_id.to_string())
}

fn upsert_blob_into_tree(
  repo: &Repository,
  base_tree: Option<&Tree<'_>>,
  path_segments: &[String],
  blob_id: git2::Oid,
) -> Result<git2::Oid, StorageError> {
  if path_segments.is_empty() {
    return Err(StorageError::InvalidFilePath(
      "file path cannot be empty".to_string(),
    ));
  }

  let mut tree_builder = repo
    .treebuilder(base_tree)
    .map_err(|err| StorageError::TreeWrite(err.to_string()))?;
  if path_segments.len() == 1 {
    tree_builder
      .insert(path_segments[0].as_str(), blob_id, 0o100644)
      .map_err(|err| StorageError::TreeWrite(err.to_string()))?;
    return tree_builder
      .write()
      .map_err(|err| StorageError::TreeWrite(err.to_string()));
  }

  let head = path_segments[0].as_str();
  let existing_subtree = resolve_subtree(repo, base_tree, head)?;
  let nested_tree_id = upsert_blob_into_tree(
    repo,
    existing_subtree.as_ref(),
    &path_segments[1..],
    blob_id,
  )?;
  tree_builder
    .insert(head, nested_tree_id, 0o040000)
    .map_err(|err| StorageError::TreeWrite(err.to_string()))?;
  tree_builder
    .write()
    .map_err(|err| StorageError::TreeWrite(err.to_string()))
}

fn resolve_subtree<'a>(
  repo: &'a Repository,
  base_tree: Option<&Tree<'a>>,
  segment: &str,
) -> Result<Option<Tree<'a>>, StorageError> {
  let Some(base_tree) = base_tree else {
    return Ok(None);
  };
  let Some(entry) = base_tree.get_name(segment) else {
    return Ok(None);
  };

  match entry.kind() {
    Some(ObjectType::Tree) => repo
      .find_tree(entry.id())
      .map(Some)
      .map_err(|err| StorageError::TreeLookup(err.to_string())),
    Some(ObjectType::Blob) => Err(StorageError::InvalidFilePath(format!(
      "path segment '{segment}' conflicts with existing file"
    ))),
    _ => Err(StorageError::InvalidFilePath(format!(
      "path segment '{segment}' is unsupported"
    ))),
  }
}

fn normalize_repo_file_path(path: &str) -> Result<Vec<String>, StorageError> {
  let normalized = path.trim().replace('\\', "/");
  let trimmed = normalized.trim_matches('/');
  if trimmed.is_empty() {
    return Err(StorageError::InvalidFilePath(
      "file path cannot be empty".to_string(),
    ));
  }

  let mut segments = Vec::new();
  for segment in trimmed.split('/') {
    if segment.is_empty() || segment == "." || segment == ".." || segment.contains('\0') {
      return Err(StorageError::InvalidFilePath(path.to_string()));
    }
    segments.push(segment.to_string());
  }

  Ok(segments)
}

fn normalize_author_name(value: &str) -> String {
  let trimmed = value.trim();
  if trimmed.is_empty() {
    return "gity".to_string();
  }
  trimmed.to_string()
}

fn normalize_author_email(value: &str) -> String {
  let trimmed = value.trim();
  if trimmed.is_empty() {
    return "noreply@gity.local".to_string();
  }
  trimmed.to_string()
}

fn is_valid_default_branch(value: &str) -> bool {
  !value.is_empty()
    && !value.contains('\\')
    && !value.contains(' ')
    && !value.contains("..")
    && !value.starts_with('/')
    && !value.ends_with('/')
}

fn is_valid_root_file_path(value: &str) -> bool {
  !value.is_empty()
    && !value.contains('\\')
    && !value.contains('/')
    && !value.contains('\0')
    && value != "."
    && value != ".."
}
