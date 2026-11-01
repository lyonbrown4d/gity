use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "project_pipelines")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub project_id: i64,
  pub iid: i64,
  pub name: String,
  pub source: String,
  pub ref_name: Option<String>,
  pub commit_sha: Option<String>,
  pub status: String,
  pub config_source: String,
  pub config_content: String,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
  pub started_at: chrono::NaiveDateTime,
  pub finished_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
