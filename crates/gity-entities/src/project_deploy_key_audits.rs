use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "project_deploy_key_audits")]
pub struct Model {
  // The DDL is keyless; application code assigns each audit row a unique revision id.
  #[sea_orm(primary_key, auto_increment = false)]
  pub revision_id: i64,
  pub operation: String,
  pub project_deploy_key_id: i64,
  pub project_id: i64,
  pub title: String,
  pub fingerprint: String,
  pub can_push: i32,
  pub created_by_user_id: i64,
  pub last_used_at: Option<chrono::NaiveDateTime>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
