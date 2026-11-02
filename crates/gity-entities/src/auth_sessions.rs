use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "auth_sessions")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub token_hash: String,
  pub user_id: i64,
  pub kind: String,
  pub expires_at: chrono::NaiveDateTime,
  pub family: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
