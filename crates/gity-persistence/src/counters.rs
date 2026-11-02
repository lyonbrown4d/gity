use gity_core::AppError;
use gity_entities::project_iid_counters::{self as counters, Column};
use sea_orm::{
  ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, TransactionTrait,
  sea_query::{Expr, ExprTrait, OnConflict},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterKind {
  Issue,
  MergeRequest,
  Milestone,
  Pipeline,
}
impl CounterKind {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Issue => "issue",
      Self::MergeRequest => "merge_request",
      Self::Milestone => "milestone",
      Self::Pipeline => "pipeline",
    }
  }
}

/// The upsert and read share one transaction, retaining the row's write lock
/// until the allocated value is read. All dialects use the same typed columns.
pub async fn next_iid(
  connection: &DatabaseConnection,
  project: i64,
  kind: CounterKind,
) -> Result<i64, AppError> {
  let transaction = connection.begin().await?;
  let now = chrono::Utc::now().naive_utc();
  counters::Entity::insert(counters::ActiveModel {
    project_id: Set(project),
    counter_name: Set(kind.as_str().to_owned()),
    current_value: Set(1),
    created_at: Set(now),
    updated_at: Set(now),
  })
  .on_conflict(
    OnConflict::columns([Column::ProjectId, Column::CounterName])
      .value(
        Column::CurrentValue,
        Expr::col((counters::Entity, Column::CurrentValue)).add(1),
      )
      .value(Column::UpdatedAt, now)
      .to_owned(),
  )
  .exec_without_returning(&transaction)
  .await?;
  let counter = counters::Entity::find_by_id((project, kind.as_str().to_owned()))
    .filter(Column::ProjectId.eq(project))
    .one(&transaction)
    .await?
    .ok_or_else(|| AppError::internal("counter missing"))?;
  transaction.commit().await?;
  Ok(counter.current_value)
}
