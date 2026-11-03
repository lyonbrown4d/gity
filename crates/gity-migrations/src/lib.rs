//! Immutable, typed database migrations shared by all deployable components.
//!
//! Apply migrations from one deployment job before starting distributed writers.
use sea_orm::{DatabaseBackend, DatabaseConnection, DbErr, TransactionTrait};
use sea_orm_migration::{MigrationTrait, MigratorTrait};

mod guard;
mod m20261102_000001_latest_schema;
mod m20261102_000002_personal_ssh_keys;
mod m20261102_000003_merge_operations;
mod m20261102_000004_oauth;
mod m20261102_000005_subscriptions;
mod m20261102_000006_discussions;
mod m20261102_000007_search_index_jobs;
mod schema;

pub use sea_orm_migration;

pub struct Migrator;

#[sea_orm_migration::async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(m20261102_000001_latest_schema::Migration),
      Box::new(m20261102_000002_personal_ssh_keys::Migration),
      Box::new(m20261102_000003_merge_operations::Migration),
      Box::new(m20261102_000004_oauth::Migration),
      Box::new(m20261102_000005_subscriptions::Migration),
      Box::new(m20261102_000006_discussions::Migration),
      Box::new(m20261102_000007_search_index_jobs::Migration),
    ]
  }
}

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
  #[error("database schema is not managed by this version; use an empty database")]
  Unmanaged,
  #[error(
    "migration history is invalid or schema initialization is incomplete; use an empty database"
  )]
  InvalidHistory,
  #[error(transparent)]
  Database(#[from] DbErr),
}

/// Initialize an empty database or apply pending official migrations.
/// Never adopts, converts, or automatically clears an unmanaged database.
pub async fn apply(db: &DatabaseConnection) -> Result<(), MigrationError> {
  guard::validate(db).await?;
  if db.get_database_backend() == DatabaseBackend::Sqlite {
    // SQLite DDL is transactional, but the framework defaults to transactions
    // only on PostgreSQL. Include both schema and ledger in one SQLite transaction.
    let transaction = db.begin().await?;
    Migrator::up(&transaction, None).await?;
    transaction.commit().await?;
  } else {
    // PostgreSQL uses framework transactions. MySQL DDL implicitly commits;
    // a failed initialization must be inspected and recreated by the operator.
    Migrator::up(db, None).await?;
  }
  Ok(())
}
