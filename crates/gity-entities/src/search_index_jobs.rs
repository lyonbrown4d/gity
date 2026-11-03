use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "search_index_jobs")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub project_id: i64,
  pub requested_generation: i64,
  pub completed_generation: i64,
  pub published_token: Option<String>,
  pub published_generation: i64,
  pub lease_token: Option<String>,
  pub lease_generation: Option<i64>,
  pub lease_expires_at: Option<chrono::NaiveDateTime>,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
