use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "project_branch_protections")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub project_id: i64,
  pub branch_name: String,
  pub rule_type: String,
  pub push_access_level: String,
  pub merge_access_level: String,
  pub require_merge_request: i32,
  pub require_pipeline_success: i32,
  pub allow_force_push: i32,
  pub allow_delete: i32,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
