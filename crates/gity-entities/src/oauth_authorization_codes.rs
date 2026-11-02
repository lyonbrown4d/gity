use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_authorization_codes")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub code_hash: String,
  pub client_id: String,
  pub user_id: i64,
  pub redirect_uri: String,
  pub code_challenge: String,
  pub scopes: String,
  pub resource: String,
  pub admin_mode: bool,
  pub expires_at: chrono::NaiveDateTime,
  pub redeemed_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
