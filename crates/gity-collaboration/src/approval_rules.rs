use crate::{domain::*, repository::CollaborationRepository};
use gity_core::AppError;
use gity_entities::project_merge_request_approval_rules as entity;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set,
  TransactionTrait,
};

impl CollaborationRepository {
  pub async fn approval_rule(&self, project: i64, key: i64) -> Result<entity::Model, AppError> {
    entity::Entity::find()
      .filter(entity::Column::ProjectId.eq(project))
      .filter(entity::Column::Id.eq(key))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("resource not found"))
  }
  pub async fn approval_rules(&self, project: i64) -> Result<Vec<entity::Model>, AppError> {
    Ok(
      entity::Entity::find()
        .filter(entity::Column::ProjectId.eq(project))
        .order_by_asc(entity::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn create_approval_rule(
    &self,
    command: ApprovalRuleCreate,
    actor: Option<i64>,
  ) -> Result<entity::Model, AppError> {
    let now = chrono::Utc::now().naive_utc();
    let tx = self.connection.begin().await?;
    let row = entity::ActiveModel {
      id: Set(gity_core::new_id()),
      project_id: Set(command.project_id),
      name: Set(command.name),
      target_branch: Set(command.target_branch),
      approvals_required: Set(command.approvals_required),
      eligible_user_ids: Set(command.eligible_user_ids),
      code_owner: Set(command.code_owner),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(&tx)
    .await?;
    gity_persistence::audit_rule(&tx, &row, "insert", actor).await?;
    tx.commit().await?;
    Ok(row)
  }
  pub async fn update_approval_rule(
    &self,
    project: i64,
    key: i64,
    changes: ApprovalRuleChanges,
    actor: Option<i64>,
  ) -> Result<entity::Model, AppError> {
    let current = self.approval_rule(project, key).await?;
    let mut active = current.into_active_model();
    if let Change::Set(value) = changes.name {
      active.name = Set(value);
    }
    if let Change::Set(value) = changes.target_branch {
      active.target_branch = Set(value);
    }
    if let Change::Set(value) = changes.approvals_required {
      active.approvals_required = Set(value);
    }
    if let Change::Set(value) = changes.eligible_user_ids {
      active.eligible_user_ids = Set(value);
    }
    if let Change::Set(value) = changes.code_owner {
      active.code_owner = Set(value);
    }
    active.updated_at = Set(chrono::Utc::now().naive_utc());
    let tx = self.connection.begin().await?;
    let row = active.update(&tx).await?;
    gity_persistence::audit_rule(&tx, &row, "update", actor).await?;
    tx.commit().await?;
    Ok(row)
  }
}

impl CollaborationRepository {
  pub async fn delete_approval_rule(
    &self,
    project: i64,
    id: i64,
    actor: Option<i64>,
  ) -> Result<(), AppError> {
    let row = self.approval_rule(project, id).await?;
    let tx = self.connection.begin().await?;
    entity::Entity::delete_by_id(row.id).exec(&tx).await?;
    gity_persistence::audit_rule(&tx, &row, "delete", actor).await?;
    tx.commit().await?;
    Ok(())
  }
}
