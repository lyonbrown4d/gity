use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "project_merge_request_approval_rules")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub project_id: i64,
  pub name: String,
  pub target_branch: String,
  pub approvals_required: i32,
  pub eligible_user_ids: String,
  pub code_owner: i32,
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
