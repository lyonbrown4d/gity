use crate::{CreateOrganization, OrganizationChanges, ProjectRepository, access};
use gity_core::{AppError, Principal};
use gity_entities::{organization_members, organizations};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, QueryOrder, Set, TransactionTrait};
impl ProjectRepository {
  pub(crate) async fn organizations(&self) -> Result<Vec<organizations::Model>, AppError> {
    Ok(
      organizations::Entity::find()
        .order_by_asc(organizations::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn organization(
    &self,
    actor: &Principal,
    id: i64,
    minimum: i64,
  ) -> Result<(organizations::Model, String), AppError> {
    access::organization(&self.connection, actor, id, minimum).await
  }
  pub(crate) async fn create_organization(
    &self,
    actor: &Principal,
    command: CreateOrganization,
  ) -> Result<organizations::Model, AppError> {
    let tx = self.connection.begin().await?;
    let now = chrono::Utc::now().naive_utc();
    let org = organizations::ActiveModel {
      id: Set(gity_core::new_id()),
      name: Set(command.name),
      path_key: Set(command.path_key.clone()),
      full_path: Set(command.path_key),
      description: Set(command.description),
      visibility: Set(command.visibility),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(&tx)
    .await?;
    organization_members::ActiveModel {
      id: Set(gity_core::new_id()),
      organization_id: Set(org.id),
      user_id: Set(actor.user_id),
      role: Set("owner".into()),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(&tx)
    .await?;
    tx.commit().await?;
    Ok(org)
  }
  pub(crate) async fn update_organization(
    &self,
    actor: &Principal,
    id: i64,
    command: OrganizationChanges,
  ) -> Result<(organizations::Model, String), AppError> {
    let tx = self.connection.begin().await?;
    access::lock_organization(
      &tx,
      if command.path_key.is_some() {
        None
      } else {
        Some(id)
      },
    )
    .await?;
    let (model, role) = access::organization(&tx, actor, id, 50).await?;
    if let Some(key) = &command.path_key {
      let next = model
        .full_path
        .rsplit_once('/')
        .map(|(parent, _)| format!("{parent}/{key}"))
        .unwrap_or_else(|| key.clone());
      if next != model.full_path {
        crate::namespace::rename(&tx, &model.full_path, &next).await?;
      }
    }
    let mut active = organizations::Entity::find_by_id(id)
      .one(&tx)
      .await?
      .ok_or_else(|| AppError::not_found("organization not found"))?
      .into_active_model();
    if let Some(value) = command.name {
      active.name = Set(value);
    }
    if let Some(value) = command.path_key {
      active.path_key = Set(value);
    }
    if let Some(value) = command.description {
      active.description = Set(value);
    }
    if let Some(value) = command.visibility {
      active.visibility = Set(value);
    }
    active.updated_at = Set(chrono::Utc::now().naive_utc());
    let model = active.update(&tx).await?;
    tx.commit().await?;
    Ok((model, role))
  }
  pub(crate) async fn delete_organization(
    &self,
    actor: &Principal,
    id: i64,
  ) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    access::lock_organization(&tx, Some(id)).await?;
    access::organization(&tx, actor, id, 50).await?;
    // The versioned schema owns organization/project child cascades.
    organizations::Entity::delete_by_id(id).exec(&tx).await?;
    tx.commit().await?;
    Ok(())
  }
}
