use gity_entities::search_index_jobs::{Column, Entity};
use gity_persistence::search_jobs::{self, Jobs};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Database, EntityTrait, IntoActiveModel, QueryFilter, Set,
};

async fn database() -> sea_orm::DatabaseConnection {
  let db = Database::connect("sqlite::memory:").await.unwrap();
  gity_migrations::apply(&db).await.unwrap();
  db
}

#[tokio::test]
async fn dirty_updates_coalesce_and_a_write_during_a_lease_remains_pending() {
  let db = database().await;
  let jobs = Jobs::new(db.clone());
  jobs.mark_dirty(41).await.unwrap();
  jobs.mark_dirty(41).await.unwrap();
  let row = Entity::find().one(&db).await.unwrap().unwrap();
  assert_eq!(row.requested_generation, 2);
  assert_eq!(row.completed_generation, 0);

  let first = jobs.claim().await.unwrap().unwrap();
  assert_eq!(first.generation, 2);
  assert!(jobs.is_current(&first).await.unwrap());
  assert!(jobs.published(&[41]).await.unwrap().is_empty());
  search_jobs::mark_dirty(&db, 41).await.unwrap();
  assert!(jobs.complete(&first).await.unwrap());
  assert_eq!(jobs.published(&[41]).await.unwrap()[&41], first.token);

  let row = Entity::find().one(&db).await.unwrap().unwrap();
  assert_eq!(row.requested_generation, 3);
  assert_eq!(row.completed_generation, 2);
  let second = jobs.claim().await.unwrap().unwrap();
  assert_eq!(second.generation, 3);
  assert!(jobs.complete(&second).await.unwrap());
  assert!(jobs.claim().await.unwrap().is_none());
}

#[tokio::test]
async fn expired_lease_is_reclaimed_and_late_completion_cannot_clear_it() {
  let db = database().await;
  let jobs = Jobs::new(db.clone());
  jobs.mark_dirty(52).await.unwrap();
  let old = jobs.claim().await.unwrap().unwrap();

  let mut row = Entity::find()
    .filter(Column::ProjectId.eq(52))
    .one(&db)
    .await
    .unwrap()
    .unwrap()
    .into_active_model();
  row.lease_expires_at = Set(Some(
    chrono::Utc::now().naive_utc() - chrono::Duration::seconds(1),
  ));
  row.update(&db).await.unwrap();

  let fresh = jobs.claim().await.unwrap().unwrap();
  assert_ne!(old.token, fresh.token);
  assert!(!jobs.complete(&old).await.unwrap());
  assert!(!jobs.is_current(&old).await.unwrap());
  let current = Entity::find()
    .filter(Column::ProjectId.eq(52))
    .one(&db)
    .await
    .unwrap()
    .unwrap();
  assert_eq!(current.lease_token.as_deref(), Some(fresh.token.as_str()));
  assert!(jobs.complete(&fresh).await.unwrap());
  assert!(jobs.claim().await.unwrap().is_none());
}
