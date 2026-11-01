use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "project_pipeline_jobs")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub project_id: i64,
  pub pipeline_id: i64,
  pub project_job_id: i64,
  pub name: String,
  pub stage: String,
  pub needs: Option<String>,
  pub image: Option<String>,
  pub script: String,
  pub artifacts: Option<String>,
  pub sort_order: i32,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
  fn new() -> Self {
    Self {
      id: sea_orm::ActiveValue::Set(gity_ids::new_id()),
      ..<Self as ActiveModelTrait>::default()
    }
  }
}
