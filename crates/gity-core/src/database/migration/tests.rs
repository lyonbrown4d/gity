use super::*;
use serde_json::json;

#[test]
fn initial_catalogs_and_component_migrations_cover_latest_schema() {
  for backend in [
    DatabaseBackend::Sqlite,
    DatabaseBackend::Postgres,
    DatabaseBackend::MySql,
  ] {
    let files = catalog(backend).unwrap();
    assert!(!files.is_empty());
    let sql = files
      .iter()
      .map(|file| file.sql.as_str())
      .collect::<Vec<_>>()
      .join("\n");
    let initial = files
      .iter()
      .take(5)
      .map(|file| file.sql.as_str())
      .collect::<Vec<_>>()
      .join("\n");
    assert_eq!(initial.matches("CREATE TABLE ").count(), 46);
    assert!(!initial.to_uppercase().contains("ALTER TABLE"));
    assert_eq!(sql.matches("CREATE TABLE ").count(), 53);
    for table in [
      "artifact_storage_deletions",
      "project_npm_dist_tags",
      "git_repository_placements",
      "project_registry_settings",
    ] {
      assert!(sql.contains(table), "missing component table {table}");
    }
    for table in ["user_passwords", "auth_sessions", "bootstrap_claim"] {
      assert!(sql.contains(table));
    }
    assert!(files.iter().all(|file| file.sql.lines().count() <= 300));
  }
}

#[tokio::test]
async fn unmanaged_tables_and_views_are_rejected_without_creating_a_ledger() {
  for ddl in [
    "CREATE TABLE unrelated_data(id INTEGER PRIMARY KEY)",
    "CREATE VIEW unrelated_view AS SELECT 1 AS value",
  ] {
    let db = Db::connect("sqlite::memory:").await.unwrap();
    db.connection.execute_unprepared(ddl).await.unwrap();
    let error = db.migrate().await.unwrap_err();
    assert_eq!(error.status.as_u16(), 409);
    assert!(error.message.contains("use an empty database"));
    assert!(
      db.one(
        "SELECT name FROM sqlite_master WHERE name='rust_schema_history'",
        vec![]
      )
      .await
      .unwrap()
      .is_none()
    );
  }
}

#[tokio::test]
async fn verified_history_prefix_advances_to_latest_without_losing_rows() {
  let db = Db::connect("sqlite::memory:").await.unwrap();
  let files = catalog(DatabaseBackend::Sqlite).unwrap();
  create_ledger(&db).await;
  db.connection
    .execute_unprepared(&files[0].sql)
    .await
    .unwrap();
  db.execute(
    "INSERT INTO rust_schema_history(name,checksum,applied_at) VALUES (?,?,?)",
    vec![
      files[0].name.clone().into(),
      files[0].checksum.clone().into(),
      now().into(),
    ],
  )
  .await
  .unwrap();
  db.insert(
    "users",
    json!({"id":42,"username":"latest-user","display_name":"Latest"})
      .as_object()
      .unwrap()
      .clone(),
  )
  .await
  .unwrap();
  db.migrate().await.unwrap();
  db.migrate().await.unwrap();
  let user = db
    .one("SELECT * FROM users WHERE id=42", vec![])
    .await
    .unwrap()
    .unwrap();
  assert_eq!(user["username"], "latest-user");
  assert_eq!(user["status"], "active");
  assert!(
    db.one("SELECT * FROM project_todos", vec![])
      .await
      .unwrap()
      .is_none()
  );
  let history = db
    .all(
      "SELECT name,checksum FROM rust_schema_history ORDER BY name",
      vec![],
    )
    .await
    .unwrap();
  assert_eq!(history.len(), files.len());
  validate_history(&files, &history).unwrap();
}

#[tokio::test]
async fn checksum_unknown_history_and_missing_prefix_fail_closed() {
  for state in ["checksum", "unknown", "gap"] {
    let db = Db::connect("sqlite::memory:").await.unwrap();
    db.migrate().await.unwrap();
    db.insert(
      "users",
      json!({"id":42,"username":"retained","display_name":"Retained"})
        .as_object()
        .unwrap()
        .clone(),
    )
    .await
    .unwrap();
    let first = &catalog(DatabaseBackend::Sqlite).unwrap()[0].name;
    match state {
      "checksum" => {
        db.execute(
          "UPDATE rust_schema_history SET checksum=? WHERE name=?",
          vec!["changed".into(), first.clone().into()],
        )
        .await
        .unwrap();
      }
      "unknown" => {
        db.execute(
          "INSERT INTO rust_schema_history(name,checksum,applied_at) VALUES (?,?,?)",
          vec![
            "sqlite/V9999__foreign.sql".into(),
            "foreign".into(),
            now().into(),
          ],
        )
        .await
        .unwrap();
      }
      _ => {
        db.execute(
          "DELETE FROM rust_schema_history WHERE name=?",
          vec![first.clone().into()],
        )
        .await
        .unwrap();
      }
    }
    assert_eq!(db.migrate().await.unwrap_err().status.as_u16(), 409);
    assert_eq!(
      db.one("SELECT username FROM users WHERE id=42", vec![])
        .await
        .unwrap()
        .unwrap()["username"],
      "retained"
    );
  }
}

#[tokio::test]
async fn empty_or_malformed_history_rejects_an_untracked_schema() {
  let db = Db::connect("sqlite::memory:").await.unwrap();
  create_ledger(&db).await;
  db.connection
    .execute_unprepared("CREATE TABLE unrelated_data(id INTEGER PRIMARY KEY)")
    .await
    .unwrap();
  assert_eq!(db.migrate().await.unwrap_err().status.as_u16(), 409);
  let malformed = Db::connect("sqlite::memory:").await.unwrap();
  malformed
    .connection
    .execute_unprepared("CREATE TABLE rust_schema_history(unrelated TEXT)")
    .await
    .unwrap();
  assert_eq!(malformed.migrate().await.unwrap_err().status.as_u16(), 409);
}
async fn create_ledger(db: &Db) {
  db.connection.execute_unprepared("CREATE TABLE rust_schema_history(name VARCHAR(255) PRIMARY KEY, checksum VARCHAR(64) NOT NULL, applied_at VARCHAR(64) NOT NULL)").await.unwrap();
}
