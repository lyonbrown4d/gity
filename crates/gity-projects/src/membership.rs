use crate::{OrganizationMemberChange, ProjectRepository, access};
use gity_core::{AppError, Principal};
use gity_entities::{organization_members, project_members, users};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, Set,
  TransactionTrait,
};
impl ProjectRepository {
  pub(crate) async fn set_member(
    &self,
    actor: &Principal,
    project: i64,
    user: i64,
    role: &str,
  ) -> Result<project_members::Model, AppError> {
    let tx = self.connection.begin().await?;
    crate::lfs::lock_project(&tx, project).await?;
    let (_, level) = access::project(&tx, actor, project, 40).await?;
    let current = project_members::Entity::find()
      .filter(project_members::Column::ProjectId.eq(project))
      .filter(project_members::Column::UserId.eq(user))
      .one(&tx)
      .await?;
    if (role == "owner" || current.as_ref().is_some_and(|row| row.role == "owner")) && level < 50 {
      return Err(AppError::forbidden("owner role required"));
    }
    users::Entity::find_by_id(user)
      .one(&tx)
      .await?
      .ok_or_else(|| AppError::not_found("user not found"))?;
    let now = chrono::Utc::now().naive_utc();
    let (member, operation) = if let Some(current) = current {
      let mut active = current.into_active_model();
      active.role = Set(role.into());
      active.updated_at = Set(now);
      (active.update(&tx).await?, "update")
    } else {
      (
        project_members::ActiveModel {
          id: Set(gity_core::new_id()),
          project_id: Set(project),
          user_id: Set(user),
          role: Set(role.into()),
          created_at: Set(now),
          updated_at: Set(now),
        }
        .insert(&tx)
        .await?,
        "insert",
      )
    };
    gity_persistence::audit_member(&tx, &member, operation, Some(actor.user_id)).await?;
    tx.commit().await?;
    Ok(member)
  }
  pub(crate) async fn delete_member(
    &self,
    actor: &Principal,
    project: i64,
    user: i64,
  ) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    crate::lfs::lock_project(&tx, project).await?;
    let (_, level) = access::project(&tx, actor, project, 40).await?;
    if let Some(member) = project_members::Entity::find()
      .filter(project_members::Column::ProjectId.eq(project))
      .filter(project_members::Column::UserId.eq(user))
      .one(&tx)
      .await?
    {
      if member.role == "owner" && level < 50 {
        return Err(AppError::forbidden("owner role required"));
      }
      project_members::Entity::delete_by_id(member.id)
        .exec(&tx)
        .await?;
      gity_persistence::audit_member(&tx, &member, "delete", Some(actor.user_id)).await?;
    }
    tx.commit().await?;
    Ok(())
  }
  pub(crate) async fn change_organization_member(
    &self,
    actor: &Principal,
    organization: i64,
    user: i64,
    change: OrganizationMemberChange<'_>,
  ) -> Result<Option<organization_members::Model>, AppError> {
    use OrganizationMemberChange::{Add, Remove, Set as Change};
    let tx = self.connection.begin().await?;
    access::lock_organization(&tx, Some(organization)).await?;
    access::organization(&tx, actor, organization, 50).await?;
    let existing = organization_members::Entity::find()
      .filter(organization_members::Column::OrganizationId.eq(organization))
      .filter(organization_members::Column::UserId.eq(user))
      .one(&tx)
      .await?;
    if matches!(change, Add(_)) && existing.is_some() {
      return Err(AppError::conflict("organization member already exists"));
    }
    if !matches!(change, Add(_)) && existing.is_none() {
      return Err(AppError::not_found("organization member not found"));
    }
    if !matches!(change, Add("owner") | Change("owner"))
      && existing.as_ref().is_some_and(|row| row.role == "owner")
    {
      let count = organization_members::Entity::find()
        .filter(organization_members::Column::OrganizationId.eq(organization))
        .filter(organization_members::Column::Role.eq("owner"))
        .count(&tx)
        .await?;
      if count <= 1 {
        return Err(AppError::conflict(
          "organization must retain at least one owner",
        ));
      }
    }
    let now = chrono::Utc::now().naive_utc();
    let member = match change {
      Add(role) => {
        users::Entity::find_by_id(user)
          .one(&tx)
          .await?
          .ok_or_else(|| AppError::not_found("user not found"))?;
        Some(
          organization_members::ActiveModel {
            id: Set(gity_core::new_id()),
            organization_id: Set(organization),
            user_id: Set(user),
            role: Set(role.into()),
            created_at: Set(now),
            updated_at: Set(now),
          }
          .insert(&tx)
          .await?,
        )
      }
      Change(role) => {
        let mut model = existing.unwrap().into_active_model();
        model.role = Set(role.into());
        model.updated_at = Set(now);
        Some(model.update(&tx).await?)
      }
      Remove => {
        organization_members::Entity::delete_by_id(existing.unwrap().id)
          .exec(&tx)
          .await?;
        None
      }
    };
    tx.commit().await?;
    Ok(member)
  }
}
