use crate::CollaborationService;
use gity_core::AppError;
use gity_entities::merge_operations as operations;
use sea_orm::{
  ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, TransactionTrait, sea_query::Expr,
};

/// Serializes Git observation/mutation and DB completion across application
/// instances. Completion uses this same transaction, including on SQLite.
pub struct MergeOperationGuard {
  pub(crate) tx: DatabaseTransaction,
  pub(crate) operation: operations::Model,
}

impl MergeOperationGuard {
  pub fn operation(&self) -> &operations::Model {
    &self.operation
  }

  /// Release an unchanged operation before any other application DB calls.
  pub async fn release(self) -> Result<operations::Model, AppError> {
    self.tx.commit().await?;
    Ok(self.operation)
  }
}

impl CollaborationService {
  pub async fn merge_operation(&self, id: i64) -> Result<operations::Model, AppError> {
    operations::Entity::find_by_id(id)
      .one(&self.repository.connection)
      .await?
      .ok_or_else(|| AppError::not_found("merge operation not found"))
  }

  pub async fn lock_merge_operation(&self, id: i64) -> Result<MergeOperationGuard, AppError> {
    let tx = self.repository.connection.begin().await?;
    // A write before the first read obtains the row lock on PostgreSQL/MySQL
    // and the writer lock on SQLite, avoiding a stale read-to-write upgrade.
    operations::Entity::update_many()
      .col_expr(
        operations::Column::UpdatedAt,
        Expr::col(operations::Column::UpdatedAt),
      )
      .filter(operations::Column::Id.eq(id))
      .exec(&tx)
      .await?;
    let operation = operations::Entity::find_by_id(id)
      .one(&tx)
      .await?
      .ok_or_else(|| AppError::not_found("merge operation not found"))?;
    Ok(MergeOperationGuard { tx, operation })
  }
}
