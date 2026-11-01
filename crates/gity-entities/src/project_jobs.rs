use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "project_jobs")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub project_id: i64,
  pub kind: String,
  pub status: String,
  pub payload: Option<String>,
  pub result: Option<String>,
  pub attempts: i32,
  pub max_attempts: i32,
  pub run_after: chrono::NaiveDateTime,
  pub locked_by: Option<String>,
  pub locked_until: chrono::NaiveDateTime,
  pub last_error: Option<String>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
  pub started_at: chrono::NaiveDateTime,
  pub finished_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
