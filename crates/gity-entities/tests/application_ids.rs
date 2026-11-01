use gity_core::{Db, new_id};
use gity_entities::{bootstrap_claim, project_iid_counters, project_member_audits, users};
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, ActiveValue};

#[test]
fn active_model_default_generates_only_entity_ids() {
  let user: users::ActiveModel = Default::default();
  let ActiveValue::Set(user_id) = user.id else {
    panic!("users default must set a Snowflake id");
  };
  assert!(user_id > 0);

  let blank = <users::ActiveModel as ActiveModelTrait>::default();
  assert!(matches!(blank.id, ActiveValue::NotSet));
  let behavior_created = <users::ActiveModel as ActiveModelBehavior>::new();
  assert!(matches!(behavior_created.id, ActiveValue::Set(id) if id > 0));

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
