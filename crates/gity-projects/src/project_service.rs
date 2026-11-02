use crate::{
  CreateOrganization, CreateProject, OrganizationChanges, OrganizationMemberChange, ProjectChanges,
  ProjectRepository, validation,
};
use gity_core::{AppError, Principal};
use gity_entities::{organization_members, organizations, project_members, projects, users};

/// Application boundary: validate commands before executing atomic repository operations.
#[derive(Clone)]
pub struct ProjectService {
  repository: ProjectRepository,
}
impl ProjectService {
  pub(crate) fn new(repository: ProjectRepository) -> Self {
    Self { repository }
  }
  pub async fn create(
    &self,
    actor: &Principal,
    command: CreateProject,
  ) -> Result<projects::Model, AppError> {
    validation::name(&command.name)?;
    validation::slug(&command.path_key)?;
    validation::visibility(&command.visibility)?;
    if command.default_branch.trim().is_empty() {
      return Err(AppError::bad_request("default_branch required"));
    }
    self.repository.create(actor, command).await
  }
  pub async fn update(
    &self,
    actor: &Principal,
    id: i64,
    command: ProjectChanges,
  ) -> Result<projects::Model, AppError> {
    if let Some(value) = &command.name {
      validation::name(value)?;
    }
    if let Some(value) = &command.visibility {
      validation::visibility(value)?;
    }
    if command
      .default_branch
      .as_ref()
      .is_some_and(|value| value.trim().is_empty())
    {
      return Err(AppError::bad_request("default_branch required"));
    }
    self.repository.update(actor, id, command).await
  }
  pub async fn compensate_creation(&self, project: i64, actor: i64) -> Result<(), AppError> {
    self.repository.compensate_creation(project, actor).await
  }
  pub async fn mark_deleted(
    &self,
    actor: &Principal,
    id: i64,
    confirmation: &str,
  ) -> Result<(), AppError> {
    self.repository.mark_deleted(actor, id, confirmation).await
  }
  pub async fn organizations(&self) -> Result<Vec<organizations::Model>, AppError> {
    self.repository.organizations().await
  }
  pub async fn organization(
    &self,
    actor: &Principal,
    id: i64,
    minimum: i64,
  ) -> Result<(organizations::Model, String), AppError> {
    self.repository.organization(actor, id, minimum).await
  }
  pub async fn create_organization(
    &self,
    actor: &Principal,
    command: CreateOrganization,
  ) -> Result<organizations::Model, AppError> {
    validation::name(&command.name)?;
    validation::slug(&command.path_key)?;
    validation::visibility(&command.visibility)?;
    self.repository.create_organization(actor, command).await
  }
  pub async fn update_organization(
    &self,
    actor: &Principal,
    id: i64,
    command: OrganizationChanges,
  ) -> Result<(organizations::Model, String), AppError> {
    if let Some(value) = &command.name {
      validation::name(value)?;
    }
    if let Some(value) = &command.path_key {
      validation::slug(value)?;
    }
    if let Some(value) = &command.visibility {
      validation::visibility(value)?;
    }
    self
      .repository
      .update_organization(actor, id, command)
      .await
  }
  pub async fn delete_organization(&self, actor: &Principal, id: i64) -> Result<(), AppError> {
    self.repository.delete_organization(actor, id).await
  }
  pub async fn set_member(
    &self,
    actor: &Principal,
    project: i64,
    user: i64,
    role: &str,
  ) -> Result<project_members::Model, AppError> {
    validate_role(role)?;
    self.repository.set_member(actor, project, user, role).await
  }
  pub async fn delete_member(
    &self,
    actor: &Principal,
    project: i64,
    user: i64,
  ) -> Result<(), AppError> {
    self.repository.delete_member(actor, project, user).await
  }
  pub async fn change_organization_member(
    &self,
    actor: &Principal,
    organization: i64,
    user: i64,
    change: OrganizationMemberChange<'_>,
  ) -> Result<Option<organization_members::Model>, AppError> {
    if let OrganizationMemberChange::Add(role) | OrganizationMemberChange::Set(role) = change {
      validate_role(role)?;
    }
    self
      .repository
      .change_organization_member(actor, organization, user, change)
      .await
  }
  pub async fn paths(&self, candidates: Vec<String>) -> Result<Vec<projects::Model>, AppError> {
    self.repository.paths(candidates).await
  }
  pub async fn active_by_path(&self, path: &str) -> Result<Option<projects::Model>, AppError> {
    self.repository.active_by_path(path).await
  }
  pub async fn list_after(
    &self,
    cursor: i64,
    limit: u64,
  ) -> Result<Vec<projects::Model>, AppError> {
    self.repository.list_after(cursor, limit).await
  }
  pub async fn member_user_ids(
    &self,
    project: i64,
    organization: i64,
    organization_only: bool,
  ) -> Result<Vec<i64>, AppError> {
    self
      .repository
      .member_user_ids(project, organization, organization_only)
      .await
  }
  pub async fn members(&self, project: i64) -> Result<Vec<project_members::Model>, AppError> {
    self.repository.members(project).await
  }
  pub async fn organization_members(
    &self,
    organization: i64,
  ) -> Result<Vec<organization_members::Model>, AppError> {
    self.repository.organization_members(organization).await
  }
  pub async fn users(&self, ids: Vec<i64>) -> Result<Vec<users::Model>, AppError> {
    self.repository.users(ids).await
  }
}
fn validate_role(role: &str) -> Result<(), AppError> {
  if gity_identity::MembershipRole::parse(role).level() == 0 {
    return Err(AppError::bad_request("invalid membership role"));
  }
  Ok(())
}

impl ProjectService {
  pub async fn search_projects_after(
    &self,
    cursor: i64,
    limit: u64,
  ) -> Result<Vec<projects::Model>, AppError> {
    self.repository.search_projects_after(cursor, limit).await
  }
  pub async fn search_projects_selected(
    &self,
    ids: &[i64],
  ) -> Result<Vec<projects::Model>, AppError> {
    self.repository.search_projects_selected(ids).await
  }
}
