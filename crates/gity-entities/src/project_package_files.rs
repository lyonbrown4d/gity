use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "project_package_files")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub project_package_version_id: i64,
  pub file_name: String,
  pub file_path: String,
  pub content_type: Option<String>,
  pub byte_size: i64,
  pub sha256: String,
  pub created_by_user_id: Option<i64>,
  pub pipeline_id: Option<i64>,
  pub job_id: Option<i64>,
  pub source: String,
  pub registry_metadata: Option<String>,
  pub storage_key: String,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
