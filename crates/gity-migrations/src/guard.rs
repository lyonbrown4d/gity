use crate::{MigrationError, Migrator};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use sea_orm_migration::{MigrationStatus, MigratorTrait, SchemaManager};

pub(super) async fn validate(db: &DatabaseConnection) -> Result<(), MigrationError> {
  let objects = schema_objects(db).await?;
  if !objects.iter().any(|name| name == "seaql_migrations") {
    return if objects.is_empty() {
      Ok(())
    } else {
      Err(MigrationError::Unmanaged)
    };
  }
  let manager = SchemaManager::new(db);
  for column in ["version", "applied_at"] {
    if !manager.has_column("seaql_migrations", column).await? {
      return Err(MigrationError::InvalidHistory);
    }
  }
  let migrations = Migrator::get_migration_with_status_read_only(db)
    .await
    .map_err(|_| MigrationError::InvalidHistory)?;
  let mut pending = false;
  let mut applied = 0;
  for migration in migrations {
    match migration.status() {
      MigrationStatus::Pending => pending = true,
      MigrationStatus::Applied if pending => return Err(MigrationError::InvalidHistory),
      MigrationStatus::Applied => applied += 1,
    }
  }
  let has_schema = objects.iter().any(|name| name != "seaql_migrations");
  if (applied == 0 && has_schema) || (applied > 0 && !has_schema) {
    return Err(MigrationError::InvalidHistory);
  }
  Ok(())
}

async fn schema_objects(db: &DatabaseConnection) -> Result<Vec<String>, MigrationError> {
  // Backend catalogs have no portable SeaQuery/Entity representation. This is
  // read-only schema ownership validation, not application/business SQL.
  let sql = match db.get_database_backend() {
    DatabaseBackend::Sqlite => {
      "SELECT name FROM sqlite_master WHERE type IN ('table','view') AND name NOT LIKE 'sqlite_%'"
    }
    DatabaseBackend::Postgres => {
      "SELECT table_name AS name FROM information_schema.tables WHERE table_schema=current_schema()"
    }
    DatabaseBackend::MySql => {
      "SELECT table_name AS name FROM information_schema.tables WHERE table_schema=DATABASE()"
    }
    _ => return Err(MigrationError::Unmanaged),
  };
  db.query_all_raw(Statement::from_string(db.get_database_backend(), sql))
    .await?
    .into_iter()
    .map(|row| row.try_get("", "name").map_err(MigrationError::from))
    .collect()
}
