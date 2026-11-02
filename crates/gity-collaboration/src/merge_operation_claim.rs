use crate::{CollaborationService, repository::CollaborationRepository};
use gity_core::AppError;
use gity_entities::{merge_operations as operations, project_merge_requests as requests};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait, sea_query::Expr,
};

/// Immutable authorization-time parameters; retries reuse the bound intent.
pub struct MergeOperationClaim {
  pub operation_key: String,
  pub project_id: i64,
  pub iid: i64,
  pub actor_user_id: i64,
  pub source_sha: String,
  pub target_sha: String,
  pub message: String,
  pub author_name: String,
  pub author_email: String,
}

impl CollaborationService {
  pub async fn claim_merge_operation(
    &self,
    command: MergeOperationClaim,
  ) -> Result<operations::Model, AppError> {
    self.repository.claim_merge_operation(command).await
  }
  pub async fn active_merge_operation(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<Option<operations::Model>, AppError> {
    let row = self.repository.merge_request(project, iid).await?;
    match row.active_merge_operation_id {
      Some(id) => Ok(
        operations::Entity::find_by_id(id)
          .one(&self.repository.connection)
          .await?,
      ),
      None => Ok(None),
    }
  }
}

impl CollaborationRepository {
  pub async fn claim_merge_operation(
    &self,
    command: MergeOperationClaim,
  ) -> Result<operations::Model, AppError> {
    let tx = self.connection.begin().await?;
    // A typed no-op write acquires a database row/write lock before reading on
    // PostgreSQL/MySQL and SQLite alike; separate server processes share it.
    requests::Entity::update_many()
      .col_expr(
        requests::Column::UpdatedAt,
        Expr::col(requests::Column::UpdatedAt),
      )
      .filter(requests::Column::ProjectId.eq(command.project_id))
      .filter(requests::Column::Iid.eq(command.iid))
      .exec(&tx)
      .await?;
    let mr = requests::Entity::find()
      .filter(requests::Column::ProjectId.eq(command.project_id))
      .filter(requests::Column::Iid.eq(command.iid))
      .one(&tx)
      .await?
      .ok_or_else(|| AppError::not_found("merge request not found"))?;
    if let Some(id) = mr.active_merge_operation_id {
      let existing = operations::Entity::find_by_id(id)
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::conflict("merge operation journal missing"))?;
      tx.commit().await?;
      return Ok(existing);
    }
    if mr.draft != 0 {
      return Err(AppError::conflict("merge request is marked as draft"));
    }
    if mr.state != "opened" {
      return Err(AppError::conflict("merge request is not opened"));
    }
    let now = chrono::Utc::now().naive_utc();
    let op = operations::ActiveModel {
      id: Set(gity_core::new_id()),
      operation_key: Set(command.operation_key),
      project_id: Set(mr.project_id),
      merge_request_id: Set(mr.id),
      actor_user_id: Set(command.actor_user_id),
      source_branch: Set(mr.source_branch),
      target_branch: Set(mr.target_branch),
      source_sha: Set(command.source_sha),
      target_sha: Set(command.target_sha),
      message: Set(command.message),
      author_name: Set(command.author_name),
      author_email: Set(command.author_email),
      status: Set("pending".into()),
      result_sha: Set(None),
      rejection: Set(None),
      rejection_actor_user_id: Set(None),
      events_delivered: Set(false),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(&tx)
    .await?;
    let claimed = requests::Entity::update_many()
      .col_expr(requests::Column::ActiveMergeOperationId, Expr::value(op.id))
      .filter(requests::Column::Id.eq(mr.id))
      .filter(requests::Column::State.eq("opened"))
      .filter(requests::Column::ActiveMergeOperationId.is_null())
      .exec(&tx)
      .await?;
    if claimed.rows_affected != 1 {
      return Err(AppError::conflict(
        "merge request changed during merge claim",
      ));
    }
    tx.commit().await?;
    Ok(op)
  }
}
