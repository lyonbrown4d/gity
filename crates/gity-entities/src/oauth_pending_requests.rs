use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_pending_requests")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub request_key: String,
  pub client_id: String,
  pub user_id: Option<i64>,
  pub redirect_uri: String,
  pub code_challenge: String,
  pub scopes: String,
  pub state: String,
  pub resource: String,
  pub nonce: Option<String>,
  pub admin_mode: bool,
  pub status: String,
  pub expires_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
