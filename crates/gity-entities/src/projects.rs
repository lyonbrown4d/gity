use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "projects")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub organization_id: i64,
  pub name: String,
  pub path_key: String,
  pub full_path: String,
  pub visibility: String,
  pub description: Option<String>,
  pub default_branch: String,
  pub status: String,
  pub deleted_at: Option<chrono::NaiveDateTime>,
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
