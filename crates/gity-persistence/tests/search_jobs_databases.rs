use gity_entities::search_index_jobs::{Column, Entity};
use gity_persistence::search_jobs::Jobs;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, IntoActiveModel,
  QueryFilter, Set,
};

async fn connect(url: &str) -> DatabaseConnection {
  tokio::time::timeout(std::time::Duration::from_secs(60), async {
    loop {
      match Database::connect(url).await {
        Ok(db) => break db,
        Err(_) => tokio::time::sleep(std::time::Duration::from_millis(250)).await,
      }
    }
  })
  .await
  .expect("database container readiness")
}

async fn exercise_namespaced_claims(db: DatabaseConnection) {
  gity_migrations::apply(&db).await.unwrap();
  let prefix = format!("{}-", "a".repeat(64));
  let jobs = Jobs::with_namespace(db.clone(), prefix.clone());

  jobs.mark_dirty(7001).await.unwrap();
  let first = jobs.claim().await.unwrap().unwrap();
  assert_eq!(first.token.len(), 101);
  assert!(first.token.starts_with(&prefix));
  assert!(jobs.is_current(&first).await.unwrap());

  jobs.mark_dirty(7001).await.unwrap();
  assert!(jobs.complete(&first).await.unwrap());
  assert_eq!(jobs.published(&[7001]).await.unwrap()[&7001], first.token);
  let next = jobs.claim().await.unwrap().unwrap();
  assert_eq!(next.generation, first.generation + 1);
  assert!(jobs.complete(&next).await.unwrap());

  jobs.mark_dirty(7001).await.unwrap();
  let expired = jobs.claim().await.unwrap().unwrap();
  let mut row = Entity::find()
    .filter(Column::ProjectId.eq(7001))
    .one(&db)
    .await
    .unwrap()
    .unwrap()
    .into_active_model();
  row.lease_expires_at = Set(Some(
    chrono::Utc::now().naive_utc() - chrono::Duration::seconds(1),
  ));
  row.update(&db).await.unwrap();

  let reclaimed = jobs.claim().await.unwrap().unwrap();
  assert_ne!(expired.token, reclaimed.token);
  assert!(reclaimed.token.starts_with(&prefix));
  assert!(!jobs.complete(&expired).await.unwrap());
  assert!(jobs.complete(&reclaimed).await.unwrap());
  db.close().await.unwrap();
}

#[tokio::test]
#[ignore = "requires Docker; starts an isolated PostgreSQL 18 Testcontainer"]
async fn postgres_namespaced_claim_and_publication() {
  let container = gity_test_support::DatabaseContainer::postgres()
    .await
    .unwrap();
  exercise_namespaced_claims(connect(container.dsn()).await).await;
}

#[tokio::test]
#[ignore = "requires Docker; starts an isolated MySQL 8.4 Testcontainer"]
async fn mysql_namespaced_claim_and_publication() {
  let container = gity_test_support::DatabaseContainer::mysql().await.unwrap();
  exercise_namespaced_claims(connect(container.dsn()).await).await;
}
