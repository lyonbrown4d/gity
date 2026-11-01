use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "project_merge_request_approval_rule_audits")]
pub struct Model {
  // The DDL is keyless; application code assigns each audit row a unique revision id.
  #[sea_orm(primary_key, auto_increment = false)]
  pub revision_id: i64,
  pub operation: String,
  pub project_merge_request_approval_rule_id: i64,
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

impl ActiveModelBehavior for ActiveModel {}
