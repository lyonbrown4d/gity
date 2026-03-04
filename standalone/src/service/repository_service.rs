use crate::configuration::cfg::Config;
use crate::security::organization_acl::{RequiredOrganizationRole, require_organization_role};
use crate::service::git_backend_service::{GitBackendError, GitBackendService};
use chrono::Utc;
use entity::{organization_members, repositories, repository_branches, repository_commits};
use mr_ulid::Ulid;
use repository::AppRepository;
use sea_orm::{DatabaseConnection, DbErr, Set, TransactionTrait};
use std::collections::HashSet;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct CreateRepositoryInput {
  pub organization_id: String,
  pub key: String,
  pub name: String,
  pub description: Option<String>,
  pub visibility: Option<String>,
  pub default_branch: Option<String>,
  pub current_user_id: String,
}

#[derive(Debug, Clone)]
pub struct CreateCommitInput {
  pub repo_id: String,
  pub branch_name: String,
  pub commit_sha: Option<String>,
  pub message: String,
  pub current_user_id: String,
}

#[derive(Debug, Clone)]
pub struct CreateBranchInput {
  pub repo_id: String,
  pub name: String,
  pub current_user_id: String,
}

#[derive(Debug, Clone)]
pub struct ListCommitsInput {
  pub repo_id: String,
  pub branch_name: Option<String>,
  pub limit: Option<u64>,
  pub current_user_id: String,
}

#[derive(Debug, Clone)]
pub struct ListBranchesInput {
  pub repo_id: String,
  pub current_user_id: String,
}

#[derive(Debug)]
pub enum RepositoryServiceError {
  BadRequest(String),
  Forbidden(String),
  NotFound(String),
  Conflict(String),
  Internal(String),
}

#[derive(Clone)]
pub struct RepositoryService {
  db_conn: DatabaseConnection,
  git_backend: GitBackendService,
  storage_enabled: bool,
  super_admin_identities: HashSet<String>,
}

impl RepositoryService {
  pub fn new(config: &Config, db_conn: DatabaseConnection, git_backend: GitBackendService) -> Self {
    Self {
      db_conn,
      git_backend,
      storage_enabled: config.storage.is_some(),
      super_admin_identities: collect_super_admin_identities(config),
    }
  }

  pub async fn create_repository(
    &self,
    input: CreateRepositoryInput,
  ) -> Result<repositories::Model, RepositoryServiceError> {
    self
      .require_owner_or_super_admin(
        input.current_user_id.as_str(),
        input.organization_id.as_str(),
      )
      .await?;

    let organization =
      AppRepository::find_active_organization_by_id(&self.db_conn, input.organization_id.as_str())
        .await
        .map_err(|err| Self::internal_error("failed to load organization", err))?
        .ok_or_else(|| RepositoryServiceError::NotFound("organization not found".to_string()))?;

    let repo_key = input.key.trim().to_string();
    if repo_key.is_empty() {
      return Err(RepositoryServiceError::BadRequest(
        "repository key is required".to_string(),
      ));
    }
    if !is_safe_storage_component(repo_key.as_str()) {
      return Err(RepositoryServiceError::BadRequest(
        "repository key contains unsupported characters".to_string(),
      ));
    }

    let repo_name = input.name.trim().to_string();
    if repo_name.is_empty() {
      return Err(RepositoryServiceError::BadRequest(
        "repository name is required".to_string(),
      ));
    }

    let exists = AppRepository::exists_active_repository_by_org_and_key(
      &self.db_conn,
      input.organization_id.as_str(),
      repo_key.as_str(),
    )
    .await
    .map_err(|err| Self::internal_error("failed to check repository key", err))?;

    if exists {
      return Err(RepositoryServiceError::Conflict(
        "repository key already exists in this organization".to_string(),
      ));
    }

    let visibility = parse_visibility(input.visibility.as_deref()).ok_or_else(|| {
      RepositoryServiceError::BadRequest(
        "visibility must be private, internal, or public".to_string(),
      )
    })?;

    let default_branch = input
      .default_branch
      .unwrap_or_else(|| "main".to_string())
      .trim()
      .to_string();
    if default_branch.is_empty() {
      return Err(RepositoryServiceError::BadRequest(
        "default_branch is required".to_string(),
      ));
    }

    let txn = self
      .db_conn
      .begin()
      .await
      .map_err(|err| Self::internal_error("failed to begin transaction", err))?;

    let repository = AppRepository::insert_repository(
      &txn,
      repositories::ActiveModel {
        organization_id: Set(input.organization_id),
        key: Set(repo_key),
        name: Set(repo_name),
        description: Set(input.description),
        visibility: Set(visibility),
        default_branch: Set(default_branch.clone()),
        created_by_user_id: Set(input.current_user_id),
        ..Default::default()
      },
    )
    .await
    .map_err(|err| Self::internal_error("failed to create repository", err))?;

    AppRepository::insert_branch(
      &txn,
      repository_branches::ActiveModel {
        repository_id: Set(repository.id.clone()),
        name: Set(default_branch),
        is_protected: Set(false),
        last_commit_sha: Set(None),
        ..Default::default()
      },
    )
    .await
    .map_err(|err| Self::internal_error("failed to create default branch", err))?;

    if self.storage_enabled {
      if let Err(err) = self
        .git_backend
        .init_bare_repository_storage(
          organization.key.as_str(),
          repository.key.as_str(),
          repository.default_branch.as_str(),
        )
        .await
      {
        let _ = txn.rollback().await;
        return Err(Self::map_git_backend_error(err));
      }
    }

    txn
      .commit()
      .await
      .map_err(|err| Self::internal_error("failed to commit transaction", err))?;

    Ok(repository)
  }

  pub async fn list_repositories(
    &self,
    user_id: &str,
    organization_id: &str,
  ) -> Result<Vec<repositories::Model>, RepositoryServiceError> {
    if !self.is_super_admin_user_id(user_id).await? {
      require_organization_role(
        &self.db_conn,
        user_id,
        organization_id,
        RequiredOrganizationRole::Member,
      )
      .await
      .map_err(Self::map_access_error)?;
    }

    AppRepository::list_active_repositories_by_org(&self.db_conn, organization_id)
      .await
      .map_err(|err| Self::internal_error("failed to list repositories", err))
  }

  pub async fn list_repositories_as_super_admin(
    &self,
    user_id: &str,
    organization_id: Option<&str>,
  ) -> Result<Vec<repositories::Model>, RepositoryServiceError> {
    if !self.is_super_admin_user_id(user_id).await? {
      return Err(RepositoryServiceError::Forbidden(
        "super admin permission is required".to_string(),
      ));
    }

    AppRepository::list_active_repositories(&self.db_conn, organization_id)
      .await
      .map_err(|err| Self::internal_error("failed to list repositories", err))
  }

  pub async fn require_repo_access(
    &self,
    user_id: &str,
    repo_id: &str,
    required: RequiredOrganizationRole,
  ) -> Result<repositories::Model, RepositoryServiceError> {
    let (repo, _) = self
      .require_repo_access_with_membership(user_id, repo_id, required)
      .await?;
    Ok(repo)
  }

  pub async fn require_repo_access_with_membership(
    &self,
    user_id: &str,
    repo_id: &str,
    required: RequiredOrganizationRole,
  ) -> Result<(repositories::Model, organization_members::Model), RepositoryServiceError> {
    let repository = AppRepository::find_active_repository_by_id(&self.db_conn, repo_id)
      .await
      .map_err(|err| Self::internal_error("failed to load repository", err))?
      .ok_or_else(|| RepositoryServiceError::NotFound("repository not found".to_string()))?;

    if self.is_super_admin_user_id(user_id).await? {
      let now = Utc::now().into();
      return Ok((
        repository.clone(),
        organization_members::Model {
          id: "super-admin-access".to_string(),
          organization_id: repository.organization_id.clone(),
          user_id: user_id.to_string(),
          role: organization_members::MemberRole::Owner,
          created_at: now,
          updated_at: now,
          deleted_at: None,
        },
      ));
    }

    let membership = require_organization_role(
      &self.db_conn,
      user_id,
      repository.organization_id.as_str(),
      required,
    )
    .await
    .map_err(Self::map_access_error)?;

    Ok((repository, membership))
  }

  pub async fn delete_repository(
    &self,
    current_user_id: &str,
    repo_id: &str,
  ) -> Result<(), RepositoryServiceError> {
    let repository = AppRepository::find_active_repository_by_id(&self.db_conn, repo_id)
      .await
      .map_err(|err| Self::internal_error("failed to load repository", err))?
      .ok_or_else(|| RepositoryServiceError::NotFound("repository not found".to_string()))?;

    self
      .require_owner_or_super_admin(current_user_id, repository.organization_id.as_str())
      .await?;

    let organization = AppRepository::find_active_organization_by_id(
      &self.db_conn,
      repository.organization_id.as_str(),
    )
    .await
    .map_err(|err| Self::internal_error("failed to load organization", err))?
    .ok_or_else(|| RepositoryServiceError::NotFound("organization not found".to_string()))?;

    let now = Utc::now().into();
    let repo_key = repository.key.clone();
    let organization_key = organization.key.clone();

    let txn = self
      .db_conn
      .begin()
      .await
      .map_err(|err| Self::internal_error("failed to begin transaction", err))?;

    let branches =
      AppRepository::list_repository_branches_by_repo_id(&txn, repository.id.as_str(), false)
        .await
        .map_err(|err| Self::internal_error("failed to load repository branches", err))?;

    for branch in branches {
      AppRepository::update_branch(&txn, branch, None, None, Some(Some(now)))
        .await
        .map_err(|err| Self::internal_error("failed to delete repository branch", err))?;
    }

    AppRepository::update_repository(&txn, repository, None, None, None, Some(Some(now)))
      .await
      .map_err(|err| Self::internal_error("failed to delete repository", err))?;

    txn
      .commit()
      .await
      .map_err(|err| Self::internal_error("failed to commit transaction", err))?;

    if self.storage_enabled
      && let Err(err) = self
        .git_backend
        .remove_repository_storage(organization_key.as_str(), repo_key.as_str())
        .await
    {
      warn!(
        organization = organization_key,
        repository = repo_key,
        error = err.to_string(),
        "repository metadata deleted but failed to remove storage directory"
      );
    }

    Ok(())
  }

  pub async fn create_commit(
    &self,
    input: CreateCommitInput,
  ) -> Result<repository_commits::Model, RepositoryServiceError> {
    let (repository, membership) = self
      .require_repo_access_with_membership(
        input.current_user_id.as_str(),
        input.repo_id.as_str(),
        RequiredOrganizationRole::Member,
      )
      .await?;

    let branch_name = input.branch_name.trim().to_string();
    if branch_name.is_empty() {
      return Err(RepositoryServiceError::BadRequest(
        "branch_name is required".to_string(),
      ));
    }

    let message = input.message.trim().to_string();
    if message.is_empty() {
      return Err(RepositoryServiceError::BadRequest(
        "message is required".to_string(),
      ));
    }

    let branch = AppRepository::find_active_branch_by_repo_and_name(
      &self.db_conn,
      repository.id.as_str(),
      branch_name.as_str(),
    )
    .await
    .map_err(|err| Self::internal_error("failed to load branch", err))?
    .ok_or_else(|| RepositoryServiceError::NotFound("branch not found".to_string()))?;

    if branch.is_protected && membership.role != organization_members::MemberRole::Owner {
      return Err(RepositoryServiceError::Forbidden(
        "only organization owner can commit to protected branch".to_string(),
      ));
    }

    let commit_sha = input
      .commit_sha
      .unwrap_or_else(|| Ulid::new().to_string().to_lowercase());

    let exists = AppRepository::exists_commit_by_repo_and_sha(
      &self.db_conn,
      repository.id.as_str(),
      commit_sha.as_str(),
    )
    .await
    .map_err(|err| Self::internal_error("failed to check commit sha", err))?;

    if exists {
      return Err(RepositoryServiceError::Conflict(
        "commit sha already exists in this repository".to_string(),
      ));
    }

    let txn = self
      .db_conn
      .begin()
      .await
      .map_err(|err| Self::internal_error("failed to begin transaction", err))?;

    let commit = AppRepository::insert_commit(
      &txn,
      repository_commits::ActiveModel {
        repository_id: Set(repository.id.clone()),
        branch_name: Set(branch_name),
        commit_sha: Set(commit_sha.clone()),
        message: Set(message),
        author_user_id: Set(input.current_user_id),
        ..Default::default()
      },
    )
    .await
    .map_err(|err| Self::internal_error("failed to insert commit", err))?;

    AppRepository::update_branch(&txn, branch, None, Some(Some(commit_sha)), None)
      .await
      .map_err(|err| Self::internal_error("failed to update branch", err))?;

    txn
      .commit()
      .await
      .map_err(|err| Self::internal_error("failed to commit transaction", err))?;

    Ok(commit)
  }

  pub async fn create_branch(
    &self,
    input: CreateBranchInput,
  ) -> Result<repository_branches::Model, RepositoryServiceError> {
    let repository = self
      .require_repo_access(
        input.current_user_id.as_str(),
        input.repo_id.as_str(),
        RequiredOrganizationRole::Member,
      )
      .await?;

    let name = input.name.trim().to_string();
    if name.is_empty() {
      return Err(RepositoryServiceError::BadRequest(
        "branch name is required".to_string(),
      ));
    }

    let exists = AppRepository::exists_active_branch_by_repo_and_name(
      &self.db_conn,
      repository.id.as_str(),
      name.as_str(),
    )
    .await
    .map_err(|err| Self::internal_error("failed to check branch name", err))?;

    if exists {
      return Err(RepositoryServiceError::Conflict(
        "branch already exists".to_string(),
      ));
    }

    AppRepository::insert_branch(
      &self.db_conn,
      repository_branches::ActiveModel {
        repository_id: Set(repository.id),
        name: Set(name),
        is_protected: Set(false),
        last_commit_sha: Set(None),
        ..Default::default()
      },
    )
    .await
    .map_err(|err| Self::internal_error("failed to create branch", err))
  }

  pub async fn list_commits(
    &self,
    input: ListCommitsInput,
  ) -> Result<Vec<repository_commits::Model>, RepositoryServiceError> {
    let repository = self
      .require_repo_access(
        input.current_user_id.as_str(),
        input.repo_id.as_str(),
        RequiredOrganizationRole::Member,
      )
      .await?;

    AppRepository::list_commits_by_repo(
      &self.db_conn,
      repository.id.as_str(),
      input.branch_name,
      input.limit.unwrap_or(50),
    )
    .await
    .map_err(|err| Self::internal_error("failed to list commits", err))
  }

  pub async fn list_branches(
    &self,
    input: ListBranchesInput,
  ) -> Result<Vec<repository_branches::Model>, RepositoryServiceError> {
    let repository = self
      .require_repo_access(
        input.current_user_id.as_str(),
        input.repo_id.as_str(),
        RequiredOrganizationRole::Member,
      )
      .await?;

    AppRepository::list_repository_branches_by_repo_id(&self.db_conn, repository.id.as_str(), false)
      .await
      .map_err(|err| Self::internal_error("failed to list branches", err))
  }

  pub async fn set_branch_protection(
    &self,
    user_id: &str,
    repo_id: &str,
    branch_name: &str,
    is_protected: bool,
  ) -> Result<repository_branches::Model, RepositoryServiceError> {
    let repository = self
      .require_repo_access(user_id, repo_id, RequiredOrganizationRole::Owner)
      .await?;

    let branch = AppRepository::find_active_branch_by_repo_and_name(
      &self.db_conn,
      repository.id.as_str(),
      branch_name,
    )
    .await
    .map_err(|err| Self::internal_error("failed to load branch", err))?
    .ok_or_else(|| RepositoryServiceError::NotFound("branch not found".to_string()))?;

    AppRepository::update_branch(&self.db_conn, branch, Some(is_protected), None, None)
      .await
      .map_err(|err| Self::internal_error("failed to update branch protection", err))
  }

  fn map_access_error(
    err: crate::security::organization_acl::AccessError,
  ) -> RepositoryServiceError {
    if err.status == axum::http::StatusCode::INTERNAL_SERVER_ERROR {
      RepositoryServiceError::Internal(err.message)
    } else {
      RepositoryServiceError::Forbidden(err.message)
    }
  }

  fn map_git_backend_error(err: GitBackendError) -> RepositoryServiceError {
    match err {
      GitBackendError::AlreadyExists(path) => RepositoryServiceError::Conflict(format!(
        "repository storage already exists at {}",
        path.to_string_lossy()
      )),
      GitBackendError::InvalidComponent(message) => RepositoryServiceError::Internal(message),
      GitBackendError::StorageNotConfigured
      | GitBackendError::InvalidRepositoryPath
      | GitBackendError::RepositoryNotFound => RepositoryServiceError::Internal(err.to_string()),
      GitBackendError::Io(message)
      | GitBackendError::Git(message)
      | GitBackendError::Db(message)
      | GitBackendError::Utf8(message) => RepositoryServiceError::Internal(message),
    }
  }

  fn internal_error(message: &str, err: DbErr) -> RepositoryServiceError {
    RepositoryServiceError::Internal(format!("{message}: {err}"))
  }

  async fn require_owner_or_super_admin(
    &self,
    user_id: &str,
    organization_id: &str,
  ) -> Result<(), RepositoryServiceError> {
    if self.is_super_admin_user_id(user_id).await? {
      return Ok(());
    }

    require_organization_role(
      &self.db_conn,
      user_id,
      organization_id,
      RequiredOrganizationRole::Owner,
    )
    .await
    .map_err(Self::map_access_error)?;
    Ok(())
  }

  async fn is_super_admin_user_id(&self, user_id: &str) -> Result<bool, RepositoryServiceError> {
    if self.super_admin_identities.is_empty() {
      return Ok(false);
    }

    let user = AppRepository::find_active_user_by_id(&self.db_conn, user_id)
      .await
      .map_err(|err| Self::internal_error("failed to load current user", err))?;
    let Some(user) = user else {
      return Ok(false);
    };

    let username = normalize_identity(user.username.as_str());
    let email = normalize_identity(user.email.as_str());
    Ok(
      self.super_admin_identities.contains(username.as_str())
        || self.super_admin_identities.contains(email.as_str()),
    )
  }
}

fn parse_visibility(value: Option<&str>) -> Option<repositories::RepositoryVisibility> {
  match value.unwrap_or("private").to_ascii_lowercase().as_str() {
    "private" => Some(repositories::RepositoryVisibility::Private),
    "internal" => Some(repositories::RepositoryVisibility::Internal),
    "public" => Some(repositories::RepositoryVisibility::Public),
    _ => None,
  }
}

fn is_safe_storage_component(value: &str) -> bool {
  !value.is_empty()
    && !value.contains('/')
    && !value.contains('\\')
    && !value.contains("..")
    && value
      .chars()
      .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}

fn collect_super_admin_identities(config: &Config) -> HashSet<String> {
  let mut identities = HashSet::new();
  if let Some(auth) = config.auth.as_ref() {
    if let Some(values) = auth.super_admins.as_ref() {
      for value in values {
        let normalized = normalize_identity(value.as_str());
        if !normalized.is_empty() {
          identities.insert(normalized);
        }
      }
    }

    if let Some(admin_username) = auth
      .admin
      .as_ref()
      .and_then(|admin| admin.username.as_ref())
    {
      let normalized = normalize_identity(admin_username.as_str());
      if !normalized.is_empty() {
        identities.insert(normalized);
      }
    }
  }

  identities
}

fn normalize_identity(value: &str) -> String {
  value.trim().to_ascii_lowercase()
}
