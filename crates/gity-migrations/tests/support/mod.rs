use gity_migrations::{Migrator, sea_orm_migration::MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

pub async fn connect(url: &str) -> DatabaseConnection {
  tokio::time::timeout(std::time::Duration::from_secs(60), async {
    loop {
      match Database::connect(url).await {
        Ok(db) => break db,
        Err(_) => tokio::time::sleep(std::time::Duration::from_millis(250)).await,
      }
    }
  })
  .await
  .expect("isolated test database readiness")
}

pub async fn assert_schema(db: DatabaseConnection) {
  gity_migrations::apply(&db).await.unwrap();
  gity_migrations::apply(&db).await.unwrap();
  let history = Migrator::get_applied_migrations(&db).await.unwrap();
  assert_eq!(history.len(), Migrator::migrations().len());
  assert_eq!(history[0].name(), "m20261102_000001_latest_schema");
  let manager = gity_migrations::sea_orm_migration::SchemaManager::new(&db);
  for table in [
    "user_ssh_keys",
    "merge_operations",
    "users",
    "auth_sessions",
    "project_milestones",
    "project_labels",
    "project_label_links",
    "git_repository_placements",
    "project_registry_settings",
  ] {
    assert!(manager.has_table(table).await.unwrap(), "missing {table}");
  }
  for column in ["merged_at", "head_sha", "milestone_id", "draft", "squash"] {
    assert!(
      manager
        .has_column("project_merge_requests", column)
        .await
        .unwrap()
    );
  }
  let backend = db.get_database_backend();
  if backend == DbBackend::MySql {
    assert_mysql_storage(&db).await;
  }
  for sql in [
    "INSERT INTO bootstrap_claim(id) VALUES (0)",
    "INSERT INTO bootstrap_claim(id) VALUES (-1)",
  ] {
    assert!(db.execute_unprepared(sql).await.is_err());
  }
  let omitted = match backend {
    DbBackend::MySql => "INSERT INTO bootstrap_claim () VALUES ()",
    _ => "INSERT INTO bootstrap_claim DEFAULT VALUES",
  };
  assert!(db.execute_unprepared(omitted).await.is_err());
  db.execute_unprepared("INSERT INTO bootstrap_claim(id) VALUES (1)")
    .await
    .unwrap();
  assert!(Migrator::down(&db, None).await.is_err());
  gity_migrations::apply(&db).await.unwrap();
  let retained = db
    .query_one_raw(Statement::from_string(
      backend,
      "SELECT id FROM bootstrap_claim",
    ))
    .await
    .unwrap()
    .unwrap();
  assert_eq!(retained.try_get::<i64>("", "id").unwrap(), 1);
  db.close().await.unwrap();
}

async fn assert_mysql_storage(db: &DatabaseConnection) {
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
    let row = db.query_one_raw(Statement::from_sql_and_values(DbBackend::MySql,
      "SELECT DATA_TYPE AS kind FROM information_schema.columns WHERE TABLE_SCHEMA=DATABASE() AND TABLE_NAME=? AND COLUMN_NAME=?",
      [table.into(), column.into()],
    )).await.unwrap().unwrap();
    assert_eq!(
      row.try_get::<String>("", "kind").unwrap(),
      "longtext",
      "{table}.{column}"
    );
  }
  let row = db.query_one_raw(Statement::from_string(DbBackend::MySql,
    "SELECT CAST(DATETIME_PRECISION AS SIGNED) AS precision_digits FROM information_schema.columns WHERE TABLE_SCHEMA=DATABASE() AND TABLE_NAME='auth_sessions' AND COLUMN_NAME='expires_at'",
  )).await.unwrap().unwrap();
  assert_eq!(row.try_get::<i64>("", "precision_digits").unwrap(), 6);
}
