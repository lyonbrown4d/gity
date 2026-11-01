use gity_entities::{organizations, project_ci_variables, project_runners, projects, users};
use gity_test_support::DatabaseContainer;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

async fn assert_entity_roundtrip(container: DatabaseContainer) {
  let db = tokio::time::timeout(std::time::Duration::from_secs(60), async {
    loop {
      match gity_core::Db::connect(container.dsn()).await {
        Ok(db) => break db,
        Err(_) => tokio::time::sleep(std::time::Duration::from_millis(250)).await,
      }
    }
  })
  .await
  .expect("test database TCP readiness");
  db.migrate().await.unwrap();
  let now = chrono::Utc::now().naive_utc();
  let user = users::ActiveModel {
    username: Set("typed-db-test".into()),
    display_name: Set("Typed database test".into()),
    email: Set(Some("typed-db@example.test".into())),
    status: Set("active".into()),
    is_super_admin: Set(1),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  }
  .insert(&db.connection)
  .await
  .expect("typed user insert and returned model");
  assert!(user.id > 1_000_000);
  assert_eq!(user.is_super_admin, 1);
  let fetched = users::Entity::find_by_id(user.id)
    .one(&db.connection)
    .await
    .unwrap()
    .unwrap();
  assert_eq!(fetched.username, "typed-db-test");
  assert_eq!(fetched.is_super_admin, 1);

  let organization = organizations::ActiveModel {
    name: Set("typed-db-org".into()),
    path_key: Set("typed-db-org".into()),
    full_path: Set("typed-db-org".into()),
    description: Set(None),
    visibility: Set("private".into()),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  }
  .insert(&db.connection)
  .await
  .expect("typed organization insert");
  let project = projects::ActiveModel {
    organization_id: Set(organization.id),
    name: Set("typed-db-project".into()),
    path_key: Set("typed-db-project".into()),
    full_path: Set("typed-db-org/typed-db-project".into()),
    visibility: Set("private".into()),
    description: Set(None),
    default_branch: Set("main".into()),
    status: Set("active".into()),
    deleted_at: Set(None),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  }
  .insert(&db.connection)
  .await
  .expect("typed project insert");
  let runner = project_runners::ActiveModel {
    project_id: Set(project.id),
    name: Set("runner".into()),
    description: Set(None),
    tags: Set(None),
    token: Set("runner-test-token".into()),
    status: Set("online".into()),
    active: Set(1),
    last_contact_at: Set(now),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  }
  .insert(&db.connection)
  .await
  .expect("typed runner insert");
  let fetched_runner = project_runners::Entity::find_by_id(runner.id)
    .one(&db.connection)
    .await
    .unwrap()
    .unwrap();
  assert_eq!(fetched_runner.active, 1);

  let variable_id = gity_ids::new_id();
  let variable = project_ci_variables::ActiveModel {
    id: Set(variable_id),
    project_id: Set(project.id),
    key: Set("MASKED".into()),
    value: Set("secret".into()),
    masked: Set(1),
    protected: Set(0),
    created_at: Set(now),
    updated_at: Set(now),
  }
  .insert(&db.connection)
  .await
  .expect("typed CI variable insert and returned model");
  let fetched_variable = project_ci_variables::Entity::find_by_id(variable.id)
    .one(&db.connection)
    .await
    .unwrap()
    .unwrap();
  assert_eq!(
    (fetched_variable.masked, fetched_variable.protected),
    (1, 0)
  );
  db.connection.close().await.unwrap();
}
#[tokio::test]
#[ignore = "requires Docker; starts an isolated PostgreSQL 18 Testcontainer"]
async fn postgres_entity_roundtrip() {
  assert_entity_roundtrip(DatabaseContainer::postgres().await.unwrap()).await;
}
#[tokio::test]
#[ignore = "requires Docker; starts an isolated MySQL 8.4 Testcontainer"]
async fn mysql_entity_roundtrip() {
  assert_entity_roundtrip(DatabaseContainer::mysql().await.unwrap()).await;
}
