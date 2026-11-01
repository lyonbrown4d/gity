use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "git_repository_placements")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub project_id: i64,
  pub primary_node: String,
  pub transition_target: Option<String>,
  pub epoch: i64,
  pub generation: i64,
  pub pending_writes: i64,
  pub state: String,
  pub replicas_json: String,
  pub version: i64,
  pub last_error: Option<String>,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
