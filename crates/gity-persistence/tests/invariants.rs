use gity_core::Db;
use gity_entities::{
  audit_revisions, organizations, project_ci_variable_audits as audits,
  project_ci_variables as variables, projects,
};
use gity_persistence::{CounterKind, audit_variable, next_iid};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
  sea_query::Table,
};

async fn fixture() -> (Db, i64) {
  let db = Db::connect("sqlite::memory:").await.unwrap();
  db.migrate().await.unwrap();
  let now = chrono::Utc::now().naive_utc();
  let org = organizations::ActiveModel {
    name: Set("Team".into()),
    path_key: Set("team".into()),
    full_path: Set("team".into()),
    visibility: Set("private".into()),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  }
  .insert(&db.connection)
  .await
  .unwrap();
  let project = projects::ActiveModel {
    organization_id: Set(org.id),
    name: Set("Project".into()),
    path_key: Set("project".into()),
    full_path: Set("team/project".into()),
    visibility: Set("private".into()),
    default_branch: Set("main".into()),
    status: Set("active".into()),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  }
  .insert(&db.connection)
  .await
  .unwrap();
  (db, project.id)
}

#[tokio::test]
async fn typed_counters_allocate_unique_values_and_isolate_counter_kind() {
  let (db, project) = fixture().await;
  let mut tasks = Vec::new();
  for _ in 0..16 {
    let connection = db.connection.clone();
    tasks.push(tokio::spawn(async move {
      next_iid(&connection, project, CounterKind::Issue)
        .await
        .unwrap()
    }));
  }
  let mut values = Vec::new();
  for task in tasks {
    values.push(task.await.unwrap());
  }
  values.sort_unstable();
  assert_eq!(values, (1..=16).collect::<Vec<_>>());
  assert_eq!(
    next_iid(&db.connection, project, CounterKind::Pipeline)
      .await
      .unwrap(),
    1
  );
  assert_eq!(
    next_iid(&db.connection, project, CounterKind::Issue)
      .await
      .unwrap(),
    17
  );
}

#[tokio::test]
async fn typed_audit_excludes_secrets_and_failure_rolls_back_the_mutation() {
  let (db, project) = fixture().await;
  let now = chrono::Utc::now().naive_utc();
  let variable = || variables::ActiveModel {
    project_id: Set(project),
    key: Set("SECRET".into()),
    value: Set("must-never-appear-in-audit".into()),
    masked: Set(1),
    protected: Set(0),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  let tx = db.connection.begin().await.unwrap();
  let model = variable().insert(&tx).await.unwrap();
  gity_core::audit_context(Some(42), audit_variable(&tx, &model, "insert", None))
    .await
    .unwrap();
  tx.commit().await.unwrap();
  let snapshot = audits::Entity::find()
    .one(&db.connection)
    .await
    .unwrap()
    .unwrap();
  assert_eq!(snapshot.project_ci_variable_id, model.id);
  assert!(
    !serde_json::to_string(&snapshot)
      .unwrap()
      .contains("must-never-appear")
  );
  let revision = audit_revisions::Entity::find_by_id(snapshot.revision_id)
    .one(&db.connection)
    .await
    .unwrap()
    .unwrap();
  assert_eq!(revision.actor.as_deref(), Some("42"));
  db.connection
    .execute(&Table::drop().table(audits::Entity).to_owned())
    .await
    .unwrap();
  let tx = db.connection.begin().await.unwrap();
  let mut pending = variable();
  pending.key = Set("ROLLBACK".into());
  let model = pending.insert(&tx).await.unwrap();
  assert!(
    audit_variable(&tx, &model, "insert", Some(42))
      .await
      .is_err()
  );
  tx.rollback().await.unwrap();
  assert!(
    variables::Entity::find()
      .filter(variables::Column::Key.eq("ROLLBACK"))
      .one(&db.connection)
      .await
      .unwrap()
      .is_none()
  );
  assert_eq!(
    audit_revisions::Entity::find()
      .all(&db.connection)
      .await
      .unwrap()
      .len(),
    1
  );
}
