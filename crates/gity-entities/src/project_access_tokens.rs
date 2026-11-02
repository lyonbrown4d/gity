use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "project_access_tokens")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub project_id: i64,
  pub kind: String,
  pub name: String,
  pub username: String,
  pub token: String,
  pub scopes: String,
  pub created_by_user_id: i64,
  pub expires_at: Option<chrono::NaiveDateTime>,
  pub revoked_at: Option<chrono::NaiveDateTime>,
  pub last_used_at: Option<chrono::NaiveDateTime>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
