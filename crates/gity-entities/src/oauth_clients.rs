use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_clients")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub client_id: String,
  pub name: String,
  pub redirect_uris: serde_json::Value,
  pub created_by: Option<i64>,
  pub is_dynamic: bool,
  pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
