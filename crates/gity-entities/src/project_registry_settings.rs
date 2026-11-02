use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "project_registry_settings")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub project_id: i64,
  pub mode: String,
  pub mirror_alias: Option<String>,
  pub mirror_repository_prefix: String,
  pub updated_by: Option<i64>,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
