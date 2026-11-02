use gity_core::Db;
use sea_orm::{ConnectionTrait, DatabaseBackend};
use serde_json::json;

#[tokio::test]
async fn fresh_latest_schema_is_versioned_idempotent_and_preserves_required_capacity() {
  let (db, cleanup) = fresh_database().await;
  db.migrate().await.unwrap();
  db.migrate().await.unwrap();
  let history = db
    .all(
      "SELECT version FROM seaql_migrations ORDER BY version",
      vec![],
    )
    .await
    .unwrap();
  assert_eq!(history.len(), 4);
  assert_eq!(history[0]["version"], "m20261102_000001_latest_schema");
  for table in [
    "user_passwords",
    "auth_sessions",
    "bootstrap_claim",
    "project_todos",
    "project_job_logs",
  ] {
    assert!(
      db.one(&format!("SELECT * FROM {table}"), vec![])
        .await
        .unwrap()
        .is_none()
    );
  }
  let fields = json!({"id":42,"username":"latest-schema-user","display_name":"Latest"})
    .as_object()
    .unwrap()
    .clone();
  let user = db.insert("users", fields).await.unwrap();
  assert_eq!(user["status"], "active");
  assert_eq!(
    db.insert(
      "users",
      json!({"id":43,"username":"latest-schema-user","display_name":"Duplicate"})
        .as_object()
        .unwrap()
        .clone()
    )
    .await
    .unwrap_err()
    .status
    .as_u16(),
    409
  );
  db.execute(
    "INSERT INTO user_passwords(user_id,password_hash) VALUES (?,?)",
    vec![42.into(), "test-hash".into()],
  )
  .await
  .unwrap();
  db.execute("INSERT INTO bootstrap_claim(id) VALUES (?)", vec![1.into()])
    .await
    .unwrap();
  db.execute(
    "INSERT INTO auth_sessions(token_hash,user_id,kind,expires_at,family) VALUES (?,?,?,?,?)",
    vec![
      "schema-session".into(),
      42.into(),
      "access".into(),
      "2050-01-01T00:00:00.123456Z".into(),
      "schema-family".into(),
    ],
  )
  .await
  .unwrap();
  let session = db
    .one(
      "SELECT expires_at AS stored_expiry FROM auth_sessions WHERE token_hash=?",
      vec!["schema-session".into()],
    )
    .await
    .unwrap()
    .unwrap();
  assert!(
    session["stored_expiry"]
      .as_str()
      .unwrap()
      .starts_with("2050-01-01T00:00:00.123456")
  );
  if db.backend() == DatabaseBackend::MySql {
    assert_mysql_capacity(&db).await;
  }
  db.migrate().await.unwrap();
  assert_eq!(
    db.one("SELECT username FROM users WHERE id=42", vec![])
      .await
      .unwrap()
      .unwrap()["username"],
    "latest-schema-user"
  );
  db.migrate().await.unwrap();
  assert_eq!(
    db.one(
      "SELECT password_hash FROM user_passwords WHERE user_id=42",
      vec![]
    )
    .await
    .unwrap()
    .unwrap()["password_hash"],
    "test-hash"
  );
  cleanup_database(db, cleanup).await;
}

#[tokio::test]
async fn nonempty_foreign_schema_is_rejected_without_mutating_its_tables() {
  let (db, cleanup) = fresh_database().await;
  db.connection
    .execute_unprepared(
      "CREATE TABLE foreign_application(id BIGINT PRIMARY KEY, payload VARCHAR(64) NOT NULL)",
    )
    .await
    .unwrap();
  db.execute(
    "INSERT INTO foreign_application(id,payload) VALUES (?,?)",
    vec![1.into(), "untouched".into()],
  )
  .await
  .unwrap();
  let error = db.migrate().await.unwrap_err();
  assert_eq!(error.status.as_u16(), 409);
  assert!(error.message.contains("use an empty database"));
  assert_eq!(
    db.one("SELECT payload FROM foreign_application WHERE id=1", vec![])
      .await
      .unwrap()
      .unwrap()["payload"],
    "untouched"
  );
  cleanup_database(db, cleanup).await;
}

async fn assert_mysql_capacity(db: &Db) {
  for (table, column) in [
    ("project_jobs", "payload"),
    ("project_jobs", "result"),
    ("project_jobs", "last_error"),
    ("project_pipeline_jobs", "needs"),
    ("project_pipeline_jobs", "script"),
    ("project_pipeline_jobs", "artifacts"),
    ("project_pipelines", "config_content"),
    ("project_job_logs", "output"),
    ("project_ci_variables", "value"),
  ] {
    let row = db.one("SELECT DATA_TYPE AS kind FROM information_schema.columns WHERE TABLE_SCHEMA=DATABASE() AND TABLE_NAME=? AND COLUMN_NAME=?", vec![table.into(), column.into()]).await.unwrap().unwrap();
    assert_eq!(row["kind"], "longtext", "{table}.{column}");
  }
  let column = db.one("SELECT DATA_TYPE AS kind,DATETIME_PRECISION AS fractional_digits FROM information_schema.columns WHERE TABLE_SCHEMA=DATABASE() AND TABLE_NAME='auth_sessions' AND COLUMN_NAME='expires_at'", vec![]).await.unwrap().unwrap();
  assert_eq!(column["kind"], "datetime");
  assert_eq!(column["fractional_digits"], 6);
}

async fn fresh_database() -> (Db, Option<(Db, String)>) {
  let Ok(url) = std::env::var("GITY_TEST_DATABASE_URL") else {
    return (Db::connect("sqlite::memory:").await.unwrap(), None);
  };
  let mut target = reqwest::Url::parse(&url).unwrap();
  assert!(matches!(
    target.scheme(),
    "postgres" | "postgresql" | "mysql"
  ));
  assert!(matches!(target.host_str(), Some("localhost" | "127.0.0.1")));
  let name = format!("gity_schema_test_{}", uuid::Uuid::new_v4().simple());
  let admin = Db::connect(&url).await.unwrap();
  admin
    .execute(&format!("CREATE DATABASE {name}"), vec![])
    .await
    .unwrap();
  target.set_path(&name);
  let db = Db::connect(target.as_str()).await.unwrap();
  (db, Some((admin, name)))
}
async fn cleanup_database(db: Db, cleanup: Option<(Db, String)>) {
  db.connection.close().await.unwrap();
  if let Some((admin, name)) = cleanup {
    admin
      .execute(&format!("DROP DATABASE {name}"), vec![])
      .await
      .unwrap();
    admin.connection.close().await.unwrap();
  }
}
