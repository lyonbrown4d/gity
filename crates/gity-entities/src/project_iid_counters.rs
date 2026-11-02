use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "project_iid_counters")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub project_id: i64,
  #[sea_orm(primary_key, auto_increment = false)]
  pub counter_name: String,
  pub current_value: i64,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
