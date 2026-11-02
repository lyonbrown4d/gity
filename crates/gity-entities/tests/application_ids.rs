use gity_core::{Db, new_id};
use gity_entities::{bootstrap_claim, project_iid_counters, project_member_audits, users};
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, ActiveValue, Set};

#[tokio::test]
async fn before_save_assigns_ids_only_on_insert_without_one() {
  let db = Db::connect("sqlite::memory:").await.unwrap();
  let connection = &db.connection;

  let blank = <users::ActiveModel as ActiveModelTrait>::default();
  assert!(matches!(blank.id, ActiveValue::NotSet));

  let inserted = <users::ActiveModel as ActiveModelBehavior>::before_save(blank, connection, true)
    .await
    .unwrap();
  assert!(matches!(inserted.id, ActiveValue::Set(id) if id > 0));

  let preset = users::ActiveModel {
    id: Set(42),
    ..Default::default()
  };
  let kept = <users::ActiveModel as ActiveModelBehavior>::before_save(preset, connection, true)
    .await
    .unwrap();
  assert!(matches!(kept.id, ActiveValue::Set(42)));

  let untouched: users::ActiveModel = Default::default();
  let updated =
    <users::ActiveModel as ActiveModelBehavior>::before_save(untouched, connection, false)
      .await
      .unwrap();
  assert!(matches!(updated.id, ActiveValue::NotSet));

  let audit = <project_member_audits::ActiveModel as ActiveModelTrait>::default();
  assert!(matches!(audit.revision_id, ActiveValue::NotSet));

  let counter = <project_iid_counters::ActiveModel as ActiveModelTrait>::default();
  assert!(matches!(counter.project_id, ActiveValue::NotSet));
  assert!(matches!(counter.counter_name, ActiveValue::NotSet));

  let claim = <bootstrap_claim::ActiveModel as ActiveModelTrait>::default();
  assert!(matches!(claim.id, ActiveValue::NotSet));
  assert!(new_id() > 0);
}

#[tokio::test]
async fn sqlite_rejects_implicit_rowid_and_accepts_explicit_claim_key() {
  let db = Db::connect("sqlite::memory:").await.unwrap();
  db.migrate().await.unwrap();

  let omitted = db
    .execute("INSERT INTO bootstrap_claim DEFAULT VALUES", vec![])
    .await;
  assert!(omitted.is_err());
  db.execute(
    "INSERT INTO bootstrap_claim(id) VALUES (?)",
    vec![1_i64.into()],
  )
  .await
  .unwrap();
}
