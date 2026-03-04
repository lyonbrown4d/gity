use crate::configuration::cfg::Config;
use chrono::Utc;
use git::{rpc, storage};
use repository::RepositoryBranchesRepository;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path as FsPath, PathBuf};
use tokio::fs;

#[derive(Debug)]
pub enum GitBackendError {
  StorageNotConfigured,
  InvalidRepositoryPath,
  RepositoryNotFound,
  InvalidComponent(String),
  AlreadyExists(PathBuf),
  Io(String),
  Git(String),
  Db(String),
  Utf8(String),
}

impl fmt::Display for GitBackendError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::StorageNotConfigured => write!(f, "storage.repo_root is not configured"),
      Self::InvalidRepositoryPath => write!(f, "invalid repository path"),
      Self::RepositoryNotFound => write!(f, "repository not found"),
      Self::InvalidComponent(message) => write!(f, "{message}"),
      Self::AlreadyExists(path) => {
        write!(
          f,
          "repository storage already exists at {}",
          path.to_string_lossy()
        )
      }
      Self::Io(message) => write!(f, "{message}"),
      Self::Git(message) => write!(f, "{message}"),
      Self::Db(message) => write!(f, "{message}"),
      Self::Utf8(message) => write!(f, "{message}"),
    }
  }
}

#[derive(Clone)]
pub struct GitBackendService {
  db_conn: DatabaseConnection,
  repo_root: Option<String>,
}

impl GitBackendService {
  pub fn new(config: &Config, db_conn: DatabaseConnection) -> Self {
    Self {
      db_conn,
      repo_root: config
        .storage
        .as_ref()
        .map(|storage| storage.repo_root.clone()),
    }
  }

  pub fn resolve_repo_path(&self, owner: &str, repo: &str) -> Result<PathBuf, GitBackendError> {
    if !is_safe_component(owner) || !is_safe_component(repo) {
      return Err(GitBackendError::InvalidRepositoryPath);
    }

    let root = self
      .repo_root
      .as_deref()
      .ok_or(GitBackendError::StorageNotConfigured)?;
    let repo_path = build_repo_path(root, owner, repo);
    if !repo_path.exists() {
      return Err(GitBackendError::RepositoryNotFound);
    }
    Ok(repo_path)
  }

  pub async fn run_stateless_rpc(
    &self,
    repo_path: &FsPath,
    service: &str,
    request_body: &[u8],
  ) -> Result<Vec<u8>, GitBackendError> {
    let repo_path = repo_path.to_path_buf();
    let service = service.to_string();
    let body = request_body.to_vec();
    let output = tokio::task::spawn_blocking(move || {
      rpc::run_stateless_rpc(repo_path.as_path(), service.as_str(), body.as_slice())
    })
    .await
    .map_err(|err| GitBackendError::Git(format!("failed to join git rpc task: {err}")))?;
    output.map_err(|err| GitBackendError::Git(err.to_string()))
  }

  pub async fn sync_branches_from_refs(
    &self,
    repository_id: &str,
    repo_path: &FsPath,
  ) -> Result<(), GitBackendError> {
    let refs = self.list_head_refs(repo_path).await?;
    let existing = RepositoryBranchesRepository::list_repository_branches_by_repo_id(
      &self.db_conn,
      repository_id,
      true,
    )
    .await
    .map_err(|err| GitBackendError::Db(format!("failed to load repository branches: {err}")))?;

    let mut existing_by_name: HashMap<String, entity::repository_branches::Model> = existing
      .into_iter()
      .map(|model| (model.name.clone(), model))
      .collect();

    let now = Utc::now().into();
    for (branch_name, commit_sha) in refs {
      if let Some(model) = existing_by_name.remove(branch_name.as_str()) {
        RepositoryBranchesRepository::update_branch(
          &self.db_conn,
          model,
          None,
          Some(Some(commit_sha)),
          Some(None),
        )
        .await
        .map_err(|err| GitBackendError::Db(format!("failed to update branch metadata: {err}")))?;
        continue;
      }

      RepositoryBranchesRepository::insert_branch(
        &self.db_conn,
        entity::repository_branches::ActiveModel {
          repository_id: sea_orm::Set(repository_id.to_string()),
          name: sea_orm::Set(branch_name),
          is_protected: sea_orm::Set(false),
          last_commit_sha: sea_orm::Set(Some(commit_sha)),
          ..Default::default()
        },
      )
      .await
      .map_err(|err| GitBackendError::Db(format!("failed to insert branch metadata: {err}")))?;
    }

    for (_, model) in existing_by_name {
      if model.deleted_at.is_some() || model.last_commit_sha.is_none() {
        continue;
      }

      RepositoryBranchesRepository::update_branch(
        &self.db_conn,
        model,
        None,
        Some(None),
        Some(Some(now)),
      )
      .await
      .map_err(|err| {
        GitBackendError::Db(format!("failed to mark deleted branch metadata: {err}"))
      })?;
    }

    Ok(())
  }

  pub async fn init_bare_repository_storage(
    &self,
    organization_key: &str,
    repository_key: &str,
    default_branch: &str,
  ) -> Result<(), GitBackendError> {
    if !is_safe_component(organization_key) {
      return Err(GitBackendError::InvalidComponent(
        "organization key contains unsupported characters".to_string(),
      ));
    }
    if !is_safe_component(repository_key) {
      return Err(GitBackendError::InvalidComponent(
        "repository key contains unsupported characters".to_string(),
      ));
    }
    if default_branch.trim().is_empty() {
      return Err(GitBackendError::InvalidComponent(
        "default branch cannot be empty".to_string(),
      ));
    }

    let root = self
      .repo_root
      .as_deref()
      .ok_or(GitBackendError::StorageNotConfigured)?;
    let repo_path = build_repo_path(root, organization_key, repository_key);
    if repo_path.exists() {
      return Err(GitBackendError::AlreadyExists(repo_path));
    }

    let parent = repo_path.parent().ok_or_else(|| {
      GitBackendError::Io(format!(
        "failed to resolve parent directory for {}",
        repo_path.to_string_lossy()
      ))
    })?;
    fs::create_dir_all(parent)
      .await
      .map_err(|err| GitBackendError::Io(format!("failed to create storage directories: {err}")))?;

    let repo_path_clone = repo_path.clone();
    let default_branch = default_branch.to_string();
    let init_result = tokio::task::spawn_blocking(move || {
      storage::init_bare_repository(repo_path_clone.as_path(), default_branch.as_str())
    })
    .await
    .map_err(|err| GitBackendError::Git(format!("failed to join git init task: {err}")))?;

    if let Err(err) = init_result {
      let _ = fs::remove_dir_all(&repo_path).await;
      return Err(GitBackendError::Git(format!(
        "failed to initialize bare repository storage: {err}"
      )));
    }

    Ok(())
  }

  pub async fn seed_initial_commit(
    &self,
    organization_key: &str,
    repository_key: &str,
    default_branch: &str,
    files: Vec<(String, String)>,
    commit_message: &str,
  ) -> Result<Option<String>, GitBackendError> {
    if files.is_empty() {
      return Ok(None);
    }
    if !is_safe_component(organization_key) {
      return Err(GitBackendError::InvalidComponent(
        "organization key contains unsupported characters".to_string(),
      ));
    }
    if !is_safe_component(repository_key) {
      return Err(GitBackendError::InvalidComponent(
        "repository key contains unsupported characters".to_string(),
      ));
    }
    if default_branch.trim().is_empty() {
      return Err(GitBackendError::InvalidComponent(
        "default branch cannot be empty".to_string(),
      ));
    }

    let root = self
      .repo_root
      .as_deref()
      .ok_or(GitBackendError::StorageNotConfigured)?;
    let repo_path = build_repo_path(root, organization_key, repository_key);
    if !repo_path.exists() {
      return Err(GitBackendError::RepositoryNotFound);
    }

    let default_branch = default_branch.to_string();
    let commit_message = commit_message.to_string();
    let repo_path_clone = repo_path.clone();
    let commit_result = tokio::task::spawn_blocking(move || {
      storage::seed_initial_commit(
        repo_path_clone.as_path(),
        default_branch.as_str(),
        files.as_slice(),
        commit_message.as_str(),
      )
    })
    .await
    .map_err(|err| GitBackendError::Git(format!("failed to join git commit task: {err}")))?;

    commit_result.map_err(|err| GitBackendError::Git(err.to_string()))
  }

  pub async fn remove_repository_storage(
    &self,
    organization_key: &str,
    repository_key: &str,
  ) -> Result<(), GitBackendError> {
    if !is_safe_component(organization_key) {
      return Err(GitBackendError::InvalidComponent(
        "organization key contains unsupported characters".to_string(),
      ));
    }
    if !is_safe_component(repository_key) {
      return Err(GitBackendError::InvalidComponent(
        "repository key contains unsupported characters".to_string(),
      ));
    }

    let root = self
      .repo_root
      .as_deref()
      .ok_or(GitBackendError::StorageNotConfigured)?;
    let repo_path = build_repo_path(root, organization_key, repository_key);
    if !repo_path.exists() {
      return Ok(());
    }

    fs::remove_dir_all(&repo_path)
      .await
      .map_err(|err| GitBackendError::Io(format!("failed to remove repository storage: {err}")))?;

    Ok(())
  }

  async fn list_head_refs(
    &self,
    repo_path: &FsPath,
  ) -> Result<Vec<(String, String)>, GitBackendError> {
    let repo_path = repo_path.to_path_buf();
    let refs = tokio::task::spawn_blocking(move || storage::list_head_refs(repo_path.as_path()))
      .await
      .map_err(|err| GitBackendError::Git(format!("failed to join ref listing task: {err}")))?;

    refs.map_err(|err| GitBackendError::Git(format!("failed to list git refs: {err}")))
  }
}

fn build_repo_path(root: &str, owner: &str, repo: &str) -> PathBuf {
  let repo_name = repo.strip_suffix(".git").unwrap_or(repo);
  FsPath::new(root)
    .join(owner)
    .join(format!("{repo_name}.git"))
}

fn is_safe_component(value: &str) -> bool {
  !value.is_empty()
    && !value.contains('/')
    && !value.contains('\\')
    && !value.contains("..")
    && value
      .chars()
      .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}
