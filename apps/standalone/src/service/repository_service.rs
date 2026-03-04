use crate::configuration::cfg::Config;
use crate::security::organization_acl::{RequiredOrganizationRole, require_organization_role};
use crate::service::git_backend_service::{GitBackendError, GitBackendService};
use chrono::Utc;
use entity::{organization_members, repositories, repository_branches, repository_commits};
use mr_ulid::Ulid;
use repository::{
  OrganizationsRepository, RepositoriesRepository, RepositoryBranchesRepository,
  RepositoryCommitsRepository, UsersRepository,
};
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
  pub gitignore_template: Option<String>,
  pub license_template: Option<String>,
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

    let organization = OrganizationsRepository::find_active_organization_by_id(
      &self.db_conn,
      input.organization_id.as_str(),
    )
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

    let exists = RepositoriesRepository::exists_active_repository_by_org_and_key(
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

    let creator_user_id = input.current_user_id.clone();
    let gitignore_content = resolve_gitignore_template(input.gitignore_template.as_deref())?;
    let license_content = resolve_license_template(input.license_template.as_deref())?;
    let mut initial_files: Vec<(String, String)> = Vec::new();
    if let Some(content) = gitignore_content {
      initial_files.push((".gitignore".to_string(), content.to_string()));
    }
    if let Some(content) = license_content {
      initial_files.push(("LICENSE".to_string(), content.to_string()));
    }
    if !initial_files.is_empty() && !self.storage_enabled {
      return Err(RepositoryServiceError::BadRequest(
        "storage backend is required when initializing .gitignore or LICENSE".to_string(),
      ));
    }

    let txn = self
      .db_conn
      .begin()
      .await
      .map_err(|err| Self::internal_error("failed to begin transaction", err))?;

    let repository = RepositoriesRepository::insert_repository(
      &txn,
      repositories::ActiveModel {
        organization_id: Set(input.organization_id),
        key: Set(repo_key),
        name: Set(repo_name),
        description: Set(input.description),
        visibility: Set(visibility),
        default_branch: Set(default_branch.clone()),
        created_by_user_id: Set(creator_user_id.clone()),
        ..Default::default()
      },
    )
    .await
    .map_err(|err| Self::internal_error("failed to create repository", err))?;

    let default_branch_model = RepositoryBranchesRepository::insert_branch(
      &txn,
      repository_branches::ActiveModel {
        repository_id: Set(repository.id.clone()),
        name: Set(default_branch.clone()),
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

      if !initial_files.is_empty() {
        let seeded_commit_sha = match self
          .git_backend
          .seed_initial_commit(
            organization.key.as_str(),
            repository.key.as_str(),
            repository.default_branch.as_str(),
            initial_files,
            "Initialize repository",
          )
          .await
        {
          Ok(commit_sha) => commit_sha,
          Err(err) => {
            let _ = txn.rollback().await;
            return Err(Self::map_git_backend_error(err));
          }
        };

        if let Some(commit_sha) = seeded_commit_sha {
          RepositoryBranchesRepository::update_branch(
            &txn,
            default_branch_model,
            None,
            Some(Some(commit_sha.clone())),
            None,
          )
          .await
          .map_err(|err| Self::internal_error("failed to update default branch", err))?;

          RepositoryCommitsRepository::insert_commit(
            &txn,
            repository_commits::ActiveModel {
              repository_id: Set(repository.id.clone()),
              branch_name: Set(default_branch),
              commit_sha: Set(commit_sha),
              message: Set("Initialize repository".to_string()),
              author_user_id: Set(creator_user_id),
              ..Default::default()
            },
          )
          .await
          .map_err(|err| Self::internal_error("failed to insert initial commit", err))?;
        }
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

    RepositoriesRepository::list_active_repositories_by_org(&self.db_conn, organization_id)
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

    RepositoriesRepository::list_active_repositories(&self.db_conn, organization_id)
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
    let repository = RepositoriesRepository::find_active_repository_by_id(&self.db_conn, repo_id)
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
    let repository = RepositoriesRepository::find_active_repository_by_id(&self.db_conn, repo_id)
      .await
      .map_err(|err| Self::internal_error("failed to load repository", err))?
      .ok_or_else(|| RepositoryServiceError::NotFound("repository not found".to_string()))?;

    self
      .require_owner_or_super_admin(current_user_id, repository.organization_id.as_str())
      .await?;

    let organization = OrganizationsRepository::find_active_organization_by_id(
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

    let branches = RepositoryBranchesRepository::list_repository_branches_by_repo_id(
      &txn,
      repository.id.as_str(),
      false,
    )
    .await
    .map_err(|err| Self::internal_error("failed to load repository branches", err))?;

    for branch in branches {
      RepositoryBranchesRepository::update_branch(&txn, branch, None, None, Some(Some(now)))
        .await
        .map_err(|err| Self::internal_error("failed to delete repository branch", err))?;
    }

    RepositoriesRepository::update_repository(&txn, repository, None, None, None, Some(Some(now)))
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

    let branch = RepositoryBranchesRepository::find_active_branch_by_repo_and_name(
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

    let exists = RepositoryCommitsRepository::exists_commit_by_repo_and_sha(
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

    let commit = RepositoryCommitsRepository::insert_commit(
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

    RepositoryBranchesRepository::update_branch(&txn, branch, None, Some(Some(commit_sha)), None)
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

    let exists = RepositoryBranchesRepository::exists_active_branch_by_repo_and_name(
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

    RepositoryBranchesRepository::insert_branch(
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

    RepositoryCommitsRepository::list_commits_by_repo(
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

    RepositoryBranchesRepository::list_repository_branches_by_repo_id(
      &self.db_conn,
      repository.id.as_str(),
      false,
    )
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

    let branch = RepositoryBranchesRepository::find_active_branch_by_repo_and_name(
      &self.db_conn,
      repository.id.as_str(),
      branch_name,
    )
    .await
    .map_err(|err| Self::internal_error("failed to load branch", err))?
    .ok_or_else(|| RepositoryServiceError::NotFound("branch not found".to_string()))?;

    RepositoryBranchesRepository::update_branch(
      &self.db_conn,
      branch,
      Some(is_protected),
      None,
      None,
    )
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

    let user = UsersRepository::find_active_user_by_id(&self.db_conn, user_id)
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

fn resolve_gitignore_template(
  value: Option<&str>,
) -> Result<Option<&'static str>, RepositoryServiceError> {
  let normalized = normalize_optional_template(value);
  match normalized.as_deref() {
    None | Some("none") => Ok(None),
    Some("rust") => Ok(Some("target/\nCargo.lock\n")),
    Some("node") => Ok(Some("node_modules/\ndist/\n.env\n")),
    Some("python") => Ok(Some("__pycache__/\n*.pyc\n.venv/\n")),
    Some("go") => Ok(Some("bin/\n*.test\ncoverage.out\n")),
    Some("java") => Ok(Some("target/\n*.class\n.idea/\n")),
    Some(other) => Err(RepositoryServiceError::BadRequest(format!(
      "unsupported gitignore template: {other}"
    ))),
  }
}

fn resolve_license_template(
  value: Option<&str>,
) -> Result<Option<&'static str>, RepositoryServiceError> {
  let normalized = normalize_optional_template(value);
  match normalized.as_deref() {
    None | Some("none") => Ok(None),
    Some("mit") => Ok(Some(
      "MIT License\n\nCopyright (c) YEAR OWNER\n\nPermission is hereby granted, free of charge, to any person obtaining a copy\nof this software and associated documentation files (the \"Software\"), to deal\nin the Software without restriction, including without limitation the rights\nto use, copy, modify, merge, publish, distribute, sublicense, and/or sell\ncopies of the Software, and to permit persons to whom the Software is\nfurnished to do so.\n\nTHE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR\nIMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,\nFITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.",
    )),
    Some("apache-2.0") => Ok(Some(
      "Apache License\nVersion 2.0, January 2004\nhttp://www.apache.org/licenses/\n\nCopyright (c) YEAR OWNER\n\nLicensed under the Apache License, Version 2.0 (the \"License\");\nyou may not use this file except in compliance with the License.\nYou may obtain a copy of the License at\n\nhttp://www.apache.org/licenses/LICENSE-2.0\n\nUnless required by applicable law or agreed to in writing, software\ndistributed under the License is distributed on an \"AS IS\" BASIS,\nWITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.",
    )),
    Some("gpl-3.0") => Ok(Some(
      "GNU GENERAL PUBLIC LICENSE\nVersion 3, 29 June 2007\n\nCopyright (C) YEAR OWNER\n\nThis program is free software: you can redistribute it and/or modify\nit under the terms of the GNU General Public License as published by\nthe Free Software Foundation, either version 3 of the License, or\n(at your option) any later version.\n\nThis program is distributed in the hope that it will be useful,\nbut WITHOUT ANY WARRANTY; without even the implied warranty of\nMERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.",
    )),
    Some(other) => Err(RepositoryServiceError::BadRequest(format!(
      "unsupported license template: {other}"
    ))),
  }
}

fn normalize_optional_template(value: Option<&str>) -> Option<String> {
  let normalized = value?.trim().to_ascii_lowercase();
  if normalized.is_empty() {
    return None;
  }
  Some(normalized)
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
