use gity_migrations::{Migrator, sea_orm_migration::prelude::*};
use sea_orm::{ConnectionTrait, Database};

struct Extended;
#[async_trait::async_trait]
impl MigratorTrait for Extended {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    let mut migrations = Migrator::migrations();
    migrations.push(Box::new(AddFeature));
    migrations
  }
}

struct Failing;
#[async_trait::async_trait]
impl MigratorTrait for Failing {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    let mut migrations = Migrator::migrations();
    migrations.push(Box::new(FailAfterDdl));
    migrations
  }
}

#[derive(DeriveIden)]
enum Feature {
  Table,
  Value,
}

struct AddFeature;
impl MigrationName for AddFeature {
  fn name(&self) -> &str {
    "m20990101_000001_feature"
  }
}
#[async_trait::async_trait]
impl MigrationTrait for AddFeature {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Feature::Table)
          .col(ColumnDef::new(Feature::Value).string().not_null())
          .to_owned(),
      )
      .await
  }
}

struct FailAfterDdl;
impl MigrationName for FailAfterDdl {
  fn name(&self) -> &str {
    "m20990101_000002_failing_feature"
  }
}
#[async_trait::async_trait]
impl MigrationTrait for FailAfterDdl {
  fn use_transaction(&self) -> Option<bool> {
    Some(true)
  }

  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    AddFeature.up(manager).await?;
    Err(DbErr::Migration("injected DDL failure".into()))
  }
}

#[tokio::test]
async fn appended_migration_runs_once_and_keeps_existing_data() {
  let db = Database::connect("sqlite::memory:").await.unwrap();
  gity_migrations::apply(&db).await.unwrap();
  db.execute_unprepared("INSERT INTO bootstrap_claim(id) VALUES (1)")
    .await
    .unwrap();
  Extended::up(&db, None).await.unwrap();
  Extended::up(&db, None).await.unwrap();
  let applied = Extended::get_applied_migrations(&db).await.unwrap();
  assert_eq!(applied.len(), Migrator::migrations().len() + 1);
  assert!(SchemaManager::new(&db).has_table("feature").await.unwrap());
  assert_eq!(
    db.query_one_raw(sea_orm::Statement::from_string(
      sea_orm::DbBackend::Sqlite,
      "SELECT id FROM bootstrap_claim",
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get::<i64>("", "id")
    .unwrap(),
    1
  );
  // A binary that does not know an applied version must not modify that database.
  assert!(gity_migrations::apply(&db).await.is_err());
  assert_eq!(
    Extended::get_applied_migrations(&db).await.unwrap().len(),
    Migrator::migrations().len() + 1
  );
}

#[tokio::test]
async fn failed_transactional_upgrade_leaves_no_ddl_or_history_and_keeps_data() {
  let db = Database::connect("sqlite::memory:").await.unwrap();
  gity_migrations::apply(&db).await.unwrap();
  db.execute_unprepared("INSERT INTO bootstrap_claim(id) VALUES (1)")
    .await
    .unwrap();
  assert!(Failing::up(&db, None).await.is_err());
  assert!(!SchemaManager::new(&db).has_table("feature").await.unwrap());
  assert_eq!(
    Migrator::get_applied_migrations(&db).await.unwrap().len(),
    Migrator::migrations().len()
  );
  gity_migrations::apply(&db).await.unwrap();
  assert!(
    db.execute_unprepared("INSERT INTO bootstrap_claim(id) VALUES (1)")
      .await
      .is_err()
  );
}
