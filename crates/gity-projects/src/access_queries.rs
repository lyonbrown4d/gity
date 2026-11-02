use crate::LockQuery;
use crate::RepositoryAccessService;
use gity_core::AppError;
use gity_entities::{
  project_access_tokens, project_audit_events as events, project_branch_protections as protections,
  project_ci_variables as variables, project_deploy_keys, project_deploy_keys as keys,
  project_lfs_locks as locks, project_lfs_objects as objects, users,
};
impl RepositoryAccessService {
  pub async fn registry_settings(
    &self,
    project: i64,
  ) -> Result<Option<gity_entities::project_registry_settings::Model>, AppError> {
    self.repository.registry_settings(project).await
  }

  pub async fn list_tokens(
    &self,
    project_id: i64,
    kind: &str,
  ) -> Result<Vec<project_access_tokens::Model>, AppError> {
    self.repository.list_tokens(project_id, kind).await
  }
  pub async fn token_by_hash(
    &self,
    digest: &str,
  ) -> Result<Option<project_access_tokens::Model>, AppError> {
    self.repository.token_by_hash(digest).await
  }
  pub async fn scoped_token(
    &self,
    project_id: i64,
    id: i64,
  ) -> Result<Option<project_access_tokens::Model>, AppError> {
    self.repository.scoped_token(project_id, id).await
  }
  pub async fn list_deploy_keys(
    &self,
    project_id: i64,
  ) -> Result<Vec<project_deploy_keys::Model>, AppError> {
    self.repository.list_deploy_keys(project_id).await
  }
  pub async fn scoped_deploy_key(
    &self,
    project_id: i64,
    id: i64,
  ) -> Result<Option<project_deploy_keys::Model>, AppError> {
    self.repository.scoped_deploy_key(project_id, id).await
  }
  pub async fn variables(&self, project: i64) -> Result<Vec<variables::Model>, AppError> {
    self.repository.variables(project).await
  }
  pub async fn lfs_object(
    &self,
    project: i64,
    oid: &str,
  ) -> Result<Option<objects::Model>, AppError> {
    self.repository.lfs_object(project, oid).await
  }
  pub async fn lfs_objects(
    &self,
    project: i64,
    cursor: i64,
    limit: u64,
  ) -> Result<Vec<objects::Model>, AppError> {
    self.repository.lfs_objects(project, cursor, limit).await
  }
  pub async fn lfs_locks(
    &self,
    project: i64,
    query: LockQuery,
  ) -> Result<Vec<locks::Model>, AppError> {
    self.repository.lfs_locks(project, query).await
  }
  pub async fn lfs_owner(&self, user: i64) -> Result<users::Model, AppError> {
    self.repository.lfs_owner(user).await
  }
  pub async fn audit_events(
    &self,
    project: i64,
    limit: u64,
  ) -> Result<Vec<events::Model>, AppError> {
    self.repository.audit_events(project, limit).await
  }
  pub async fn protections(&self, project: i64) -> Result<Vec<protections::Model>, AppError> {
    self.repository.protections(project).await
  }
  pub async fn authorized_keys(&self, fingerprint: &str) -> Result<Vec<keys::Model>, AppError> {
    self.repository.authorized_keys(fingerprint).await
  }
}
