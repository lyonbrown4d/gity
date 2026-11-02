use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_tokens")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub token_hash: String,
  pub family_key: String,
  pub client_id: String,
  pub user_id: i64,
  pub kind: String,
  pub scopes: String,
  pub resource: String,
  pub admin_mode: bool,
  pub expires_at: chrono::NaiveDateTime,
  pub revoked_at: Option<chrono::NaiveDateTime>,
  pub replaced_by_hash: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
