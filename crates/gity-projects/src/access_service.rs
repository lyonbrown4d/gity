use crate::{
  AuditRecord, NewProjectToken, ProtectionCommand, RepositoryAccessRepository, VariableCommand,
};
use gity_core::AppError;
use gity_entities::{
  project_access_tokens, project_branch_protections as protections,
  project_ci_variables as variables, project_deploy_keys, project_lfs_locks as locks,
  project_lfs_objects as objects,
};
/// Application service for repository credentials and access metadata.
#[derive(Clone)]
pub struct RepositoryAccessService {
  pub(crate) repository: RepositoryAccessRepository,
}
impl RepositoryAccessService {
  pub(crate) fn new(repository: RepositoryAccessRepository) -> Self {
    Self { repository }
  }
  pub async fn set_registry_settings(
    &self,
    project: i64,
    mode: &str,
    alias: Option<String>,
    prefix: String,
    actor: i64,
  ) -> Result<(), AppError> {
    if !["hosted", "mirror"].contains(&mode) {
      return Err(AppError::bad_request("invalid registry mode"));
    }
    self
      .repository
      .set_registry_settings(project, mode, alias, prefix, actor)
      .await
  }
  pub async fn create_token(
    &self,
    input: NewProjectToken,
  ) -> Result<project_access_tokens::Model, AppError> {
    if input.name.trim().is_empty() {
      return Err(AppError::bad_request("name is required"));
    }
    if !["deploy", "project_access"].contains(&input.kind.as_str()) {
      return Err(AppError::bad_request("invalid token kind"));
    }
    if input.scopes.is_empty()
      || input.scopes.split(',').any(|scope| {
        ![
          "read_repository",
          "write_repository",
          "read_package",
          "write_package",
          "read_registry",
          "write_registry",
        ]
        .contains(&scope)
          && !(input.kind == "project_access" && ["read_api", "write_api"].contains(&scope))
      })
    {
      return Err(AppError::bad_request("token scope is invalid"));
    }
    self.repository.create_token(input).await
  }
  pub async fn mark_token_used(&self, token: project_access_tokens::Model) -> Result<(), AppError> {
    self.repository.mark_token_used(token).await
  }
  pub async fn revoke_token(&self, token: project_access_tokens::Model) -> Result<(), AppError> {
    self.repository.revoke_token(token).await
  }
  pub async fn create_deploy_key(
    &self,
    project_id: i64,
    title: String,
    public_key: String,
    fingerprint: String,
    can_push: bool,
    created_by_user_id: i64,
  ) -> Result<project_deploy_keys::Model, AppError> {
    self
      .repository
      .create_deploy_key(
        project_id,
        title,
        public_key,
        fingerprint,
        can_push,
        created_by_user_id,
      )
      .await
  }
  pub async fn delete_deploy_key(&self, key: project_deploy_keys::Model) -> Result<(), AppError> {
    self.repository.delete_deploy_key(key).await
  }
  pub async fn save_variable(
    &self,
    project: i64,
    command: VariableCommand,
  ) -> Result<variables::Model, AppError> {
    crate::validation::variable(&command)?;
    self.repository.save_variable(project, command).await
  }
  pub async fn delete_variable(&self, project: i64, key: &str) -> Result<(), AppError> {
    self.repository.delete_variable(project, key).await
  }
  pub async fn save_lfs_object(
    &self,
    project: i64,
    oid: &str,
    size: i64,
    key: String,
    missing_storage_key: Option<&str>,
  ) -> Result<objects::Model, AppError> {
    if oid.len() != 64 || !oid.bytes().all(|b| b.is_ascii_hexdigit()) {
      return Err(AppError::bad_request(
        "lfs oid must be a 64-character hex string",
      ));
    }
    if size < 0 {
      return Err(AppError::bad_request("invalid lfs object size"));
    }
    self
      .repository
      .save_lfs_object(project, oid, size, key, missing_storage_key)
      .await
  }
  pub async fn create_lfs_lock(
    &self,
    project: i64,
    user: i64,
    path: String,
  ) -> Result<locks::Model, AppError> {
    let path = crate::validation::lock_path(&path)?;
    self.repository.create_lfs_lock(project, user, path).await
  }
  pub async fn delete_lfs_lock(
    &self,
    project: i64,
    user: i64,
    id: i64,
    force: bool,
  ) -> Result<locks::Model, AppError> {
    if id <= 0 {
      return Err(AppError::bad_request("invalid lock id"));
    }
    self
      .repository
      .delete_lfs_lock(project, user, id, force)
      .await
  }
  pub async fn record_event(&self, record: AuditRecord<'_>) -> Result<(), AppError> {
    self.repository.record_event(record).await
  }
  pub async fn save_protection(
    &self,
    project: i64,
    branch: &str,
    command: &ProtectionCommand,
  ) -> Result<protections::Model, AppError> {
    if branch.is_empty() || branch.len() > 255 {
      return Err(AppError::bad_request("invalid branch pattern"));
    }
    for level in [&command.push_access_level, &command.merge_access_level] {
      if !["maintainer", "developer", "owner", "no_one"].contains(&level.as_str()) {
        return Err(AppError::bad_request(
          "invalid protected branch access level",
        ));
      }
    }
    self
      .repository
      .save_protection(project, branch, command)
      .await
  }
  pub async fn delete_protection(&self, project: i64, branch: &str) -> Result<bool, AppError> {
    self.repository.delete_protection(project, branch).await
  }
  pub async fn touch_deploy_key(&self, key: i64) -> Result<(), AppError> {
    self.repository.touch_deploy_key(key).await
  }
}
