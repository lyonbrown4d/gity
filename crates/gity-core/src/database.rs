use crate::AppError;
#[cfg(any(test, feature = "test-fixtures"))]
use crate::{new_id, now};
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection};
#[cfg(any(test, feature = "test-fixtures"))]
use sea_orm::{DatabaseTransaction, FromQueryResult, Statement, TransactionTrait, Value};
#[cfg(any(test, feature = "test-fixtures"))]
use serde_json::{Map, Value as Json};

#[derive(Clone)]
pub struct Db {
  pub connection: DatabaseConnection,
}
mod audit;
#[cfg(any(test, feature = "test-fixtures"))]
mod bindings;
#[cfg(any(test, feature = "test-fixtures"))]
mod fixtures;
#[cfg(any(test, feature = "test-fixtures"))]
pub use fixtures::sql_value;
mod migration;
pub use audit::{actor as audit_actor, context as audit_context};
impl Db {
  pub async fn connect(url: &str) -> Result<Self, AppError> {
    if let Some(path) = url
      .strip_prefix("sqlite://")
      .and_then(|s| s.split('?').next())
      && let Some(parent) = std::path::Path::new(path).parent()
    {
      tokio::fs::create_dir_all(parent).await?;
    }
    let connection = Database::connect(url).await?;
    if connection.get_database_backend() == DatabaseBackend::Sqlite {
      connection
        .execute_unprepared("PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;")
        .await?;
    }
    Ok(Self { connection })
  }
  pub fn backend(&self) -> DatabaseBackend {
    self.connection.get_database_backend()
  }
}

#[cfg(test)]
mod tests;
