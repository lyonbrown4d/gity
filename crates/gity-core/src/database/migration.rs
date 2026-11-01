use super::*;

struct Migration {
  name: String,
  sql: String,
  checksum: String,
}
impl Db {
  /// Apply this version's initial schema or advance a database with a verified
  /// migration history. Unmanaged databases are never adopted or converted.
  pub async fn migrate(&self) -> Result<(), AppError> {
    let files = catalog(self.backend())?;
    let objects = self.schema_objects().await?;
    let tracked = objects.iter().any(|name| name == "rust_schema_history");
    if !tracked && !objects.is_empty() {
      return Err(AppError::conflict(
        "database schema is not managed by this version; use an empty database",
      ));
    }
    if !tracked {
      self
        .connection
        .execute_unprepared(concat!(
          "CREATE TABLE rust_schema_history (name VARCHAR(255) PRIMARY KEY, ",
          "checksum VARCHAR(64) NOT NULL, applied_at VARCHAR(64) NOT NULL)"
        ))
        .await?;
    }
    let history = self
      .all(
        "SELECT name,checksum FROM rust_schema_history ORDER BY name",
        vec![],
      )
      .await
      .map_err(|_| AppError::conflict("migration history is invalid; use an empty database"))?;
    validate_history(&files, &history)?;
    if history.is_empty() && objects.iter().any(|name| name != "rust_schema_history") {
      return Err(AppError::conflict(
        "database contains an untracked schema; use an empty database",
      ));
    }
    if !history.is_empty() && objects.len() == 1 {
      return Err(AppError::conflict(
        "migration history exists without the application schema",
      ));
    }
    for migration in files.into_iter().skip(history.len()) {
      let tx = self.transaction().await?;
      tx.execute_unprepared(&migration.sql).await?;
      tx.execute_raw(self.statement(
        "INSERT INTO rust_schema_history(name,checksum,applied_at) VALUES (?,?,?)",
        vec![
          migration.name.into(),
          migration.checksum.into(),
          now().into(),
        ],
      ))
      .await?;
      tx.commit().await?;
    }
    Ok(())
  }
  async fn schema_objects(&self) -> Result<Vec<String>, AppError> {
    let sql = match self.backend() {
      DatabaseBackend::Sqlite => {
        "SELECT name FROM sqlite_master WHERE type IN ('table','view') AND name NOT LIKE 'sqlite_%'"
      }
      DatabaseBackend::Postgres => {
        "SELECT table_name::text AS name FROM information_schema.tables WHERE table_schema=current_schema()"
      }
      DatabaseBackend::MySql => {
        "SELECT table_name AS name FROM information_schema.tables WHERE table_schema=DATABASE()"
      }
      _ => return Err(AppError::bad_request("unsupported database")),
    };
    Ok(
      self
        .all(sql, vec![])
        .await?
        .into_iter()
        .filter_map(|row| row["name"].as_str().map(str::to_owned))
        .collect(),
    )
  }
}
fn catalog(backend: DatabaseBackend) -> Result<Vec<Migration>, AppError> {
  let dialect = match backend {
    DatabaseBackend::Sqlite => "sqlite",
    DatabaseBackend::Postgres => "postgres",
    DatabaseBackend::MySql => "mysql",
    _ => return Err(AppError::bad_request("unsupported database")),
  };
  let mut names = Migrations::iter()
    .filter(|name| name.starts_with(&format!("{dialect}/")) && name.ends_with(".sql"))
    .collect::<Vec<_>>();
  names.sort();
  names
    .into_iter()
    .map(|name| {
      let file = Migrations::get(&name).expect("embedded migration");
      Ok(Migration {
        name: name.into_owned(),
        sql: std::str::from_utf8(file.data.as_ref())
          .map_err(|_| AppError::internal("invalid migration encoding"))?
          .to_owned(),
        checksum: hex::encode(Sha256::digest(file.data.as_ref())),
      })
    })
    .collect()
}
fn validate_history(files: &[Migration], history: &[Json]) -> Result<(), AppError> {
  if history.len() > files.len()
    || history
      .iter()
      .zip(files)
      .any(|(row, file)| row["name"] != file.name)
  {
    return Err(AppError::conflict(
      "migration history does not match this version; use an empty database",
    ));
  }
  for (row, file) in history.iter().zip(files) {
    if row["checksum"] != file.checksum {
      return Err(AppError::conflict(format!(
        "migration checksum changed: {}",
        file.name
      )));
    }
  }
  Ok(())
}
#[cfg(test)]
mod tests;
