use super::*;
use serde_json::json;

#[tokio::test]
async fn migration_uses_official_history_and_preserves_application_data() {
  let db = Db::connect("sqlite::memory:").await.unwrap();
  db.migrate().await.unwrap();
  let history = db
    .all(
      "SELECT version FROM seaql_migrations ORDER BY version",
      vec![],
    )
    .await
    .unwrap();
  assert_eq!(history.len(), 1);
  assert_eq!(history[0]["version"], "m20261102_000001_latest_schema");

  db.insert(
    "users",
    json!({"id":42,"username":"migration-user","display_name":"Migration"})
      .as_object()
      .unwrap()
      .clone(),
  )
  .await
  .unwrap();
  db.migrate().await.unwrap();
  assert_eq!(
    db.one("SELECT username FROM users WHERE id=42", vec![])
      .await
      .unwrap()
      .unwrap()["username"],
    "migration-user"
  );
  let history = db
    .all(
      "SELECT version FROM seaql_migrations ORDER BY version",
      vec![],
    )
    .await
    .unwrap();
  assert_eq!(history.len(), 1);
}

#[tokio::test]
async fn preexisting_or_legacy_schema_is_rejected_without_mutation() {
  for ddl in [
    "CREATE TABLE unrelated_data(id INTEGER PRIMARY KEY)",
    "CREATE TABLE rust_schema_history(name TEXT PRIMARY KEY, checksum TEXT)",
  ] {
    let db = Db::connect("sqlite::memory:").await.unwrap();
    db.connection.execute_unprepared(ddl).await.unwrap();
    let error = db.migrate().await.unwrap_err();
    assert_eq!(error.status.as_u16(), 409);
    assert!(error.message.contains("use an empty database"));
    assert!(
      db.one(
        "SELECT name FROM sqlite_master WHERE name='seaql_migrations'",
        vec![]
      )
      .await
      .unwrap()
      .is_none()
    );
  }
}

#[tokio::test]
async fn unknown_official_history_version_fails_closed_and_retains_rows() {
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
  db.execute(
    "INSERT INTO seaql_migrations(version,applied_at) VALUES (?,?)",
    vec!["m19000101_000001_foreign_migration".into(), 0_i64.into()],
  )
  .await
  .unwrap();
  assert_eq!(db.migrate().await.unwrap_err().status.as_u16(), 409);
  assert_eq!(
    db.one("SELECT username FROM users WHERE id=42", vec![])
      .await
      .unwrap()
      .unwrap()["username"],
    "retained"
  );
}

#[tokio::test]
async fn views_and_incomplete_or_malformed_history_fail_closed() {
  let view = Db::connect("sqlite::memory:").await.unwrap();
  view
    .connection
    .execute_unprepared("CREATE VIEW unrelated_view AS SELECT 1 AS value")
    .await
    .unwrap();
  assert_eq!(view.migrate().await.unwrap_err().status.as_u16(), 409);

  let malformed = Db::connect("sqlite::memory:").await.unwrap();
  malformed
    .connection
    .execute_unprepared("CREATE TABLE seaql_migrations(unexpected TEXT)")
    .await
    .unwrap();
  assert_eq!(malformed.migrate().await.unwrap_err().status.as_u16(), 409);

  let empty = Db::connect("sqlite::memory:").await.unwrap();
  empty
    .connection
    .execute_unprepared(
      "CREATE TABLE seaql_migrations(version VARCHAR(255) PRIMARY KEY NOT NULL, applied_at BIGINT NOT NULL); CREATE TABLE unrelated_data(id INTEGER PRIMARY KEY, payload TEXT NOT NULL)",
    )
    .await
    .unwrap();
  empty
    .execute(
      "INSERT INTO unrelated_data(id,payload) VALUES (?,?)",
      vec![1.into(), "untouched".into()],
    )
    .await
    .unwrap();
  assert_eq!(empty.migrate().await.unwrap_err().status.as_u16(), 409);
  assert_eq!(
    empty
      .one("SELECT payload FROM unrelated_data WHERE id=1", vec![])
      .await
      .unwrap()
      .unwrap()["payload"],
    "untouched"
  );

  let ledger_only = Db::connect("sqlite::memory:").await.unwrap();
  ledger_only
    .connection
    .execute_unprepared(
      "CREATE TABLE seaql_migrations(version VARCHAR(255) PRIMARY KEY NOT NULL, applied_at BIGINT NOT NULL)",
    )
    .await
    .unwrap();
  ledger_only
    .execute(
      "INSERT INTO seaql_migrations(version,applied_at) VALUES (?,?)",
      vec!["m20261102_000001_latest_schema".into(), 0_i64.into()],
    )
    .await
    .unwrap();
  assert_eq!(
    ledger_only.migrate().await.unwrap_err().status.as_u16(),
    409
  );
}
