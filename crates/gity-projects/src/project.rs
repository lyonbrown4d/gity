use crate::{CreateProject, ProjectChanges, ProjectRepository, access};
use gity_core::{AppError, Principal};
use gity_entities::{project_members, projects};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set, TransactionTrait,
};

impl ProjectRepository {
  pub(crate) async fn create(
    &self,
    actor: &Principal,
    command: CreateProject,
  ) -> Result<projects::Model, AppError> {
    let tx = self.connection.begin().await?;
    access::lock_organization(&tx, Some(command.organization_id)).await?;
    let (organization, _) = access::organization(&tx, actor, command.organization_id, 30).await?;
    let now = chrono::Utc::now().naive_utc();
    let project = projects::ActiveModel {
      id: Set(gity_core::new_id()),
      organization_id: Set(command.organization_id),
      name: Set(command.name),
      path_key: Set(command.path_key.clone()),
      full_path: Set(format!("{}/{}", organization.full_path, command.path_key)),
      visibility: Set(command.visibility),
      description: Set(command.description),
      default_branch: Set(command.default_branch),
      status: Set("active".into()),
      deleted_at: Set(None),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(&tx)
    .await?;
    let member = project_members::ActiveModel {
      id: Set(gity_core::new_id()),
      project_id: Set(project.id),
      user_id: Set(actor.user_id),
      role: Set("owner".into()),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(&tx)
    .await?;
    gity_persistence::audit_member(&tx, &member, "insert", Some(actor.user_id)).await?;
    tx.commit().await?;
    Ok(project)
  }
  pub(crate) async fn compensate_creation(&self, project: i64, actor: i64) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    let members = project_members::Entity::find()
      .filter(project_members::Column::ProjectId.eq(project))
      .all(&tx)
      .await?;
    for member in members {
      gity_persistence::audit_member(&tx, &member, "delete", Some(actor)).await?;
    }
    project_members::Entity::delete_many()
      .filter(project_members::Column::ProjectId.eq(project))
      .exec(&tx)
      .await?;
    projects::Entity::delete_by_id(project).exec(&tx).await?;
    tx.commit().await?;
    Ok(())
  }
  pub(crate) async fn update(
    &self,
    actor: &Principal,
    id: i64,
    command: ProjectChanges,
  ) -> Result<projects::Model, AppError> {
    let tx = self.connection.begin().await?;
    crate::lfs::lock_project(&tx, id).await?;
    let (model, _) = access::project(&tx, actor, id, 40).await?;
    let mut active = model.into_active_model();
    if let Some(value) = command.name {
      active.name = Set(value);
    }
    if let Some(value) = command.description {
      active.description = Set(value);
    }
    if let Some(value) = command.visibility {
      active.visibility = Set(value);
    }
    if let Some(value) = command.default_branch {
      active.default_branch = Set(value);
    }
    active.updated_at = Set(chrono::Utc::now().naive_utc());
    let model = active.update(&tx).await?;
    tx.commit().await?;
    Ok(model)
  }
  pub(crate) async fn mark_deleted(
    &self,
    actor: &Principal,
    id: i64,
    confirmation: &str,
  ) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    crate::lfs::lock_project(&tx, id).await?;
    let (model, _) = access::project(&tx, actor, id, 50).await?;
    if confirmation.trim() != model.full_path {
      return Err(AppError::bad_request(
        "project delete confirmation does not match full path",
      ));
    }
    let mut active = model.into_active_model();
    let now = chrono::Utc::now().naive_utc();
    active.deleted_at = Set(Some(now));
    active.status = Set("pending_delete".into());
    active.updated_at = Set(now);
    active.update(&tx).await?;
    tx.commit().await?;
    Ok(())
  }
}
