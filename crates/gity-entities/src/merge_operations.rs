use sea_orm::entity::prelude::*;

/// Durable merge intent and completion outbox; IDs are application generated.
#[derive(Clone, Debug, PartialEq, serde::Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "merge_operations")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub operation_key: String,
  pub project_id: i64,
  pub merge_request_id: i64,
  pub actor_user_id: i64,
  pub source_branch: String,
  pub target_branch: String,
  pub source_sha: String,
  pub target_sha: String,
  pub message: String,
  pub author_name: String,
  pub author_email: String,
  pub status: String,
  pub result_sha: Option<String>,
  pub rejection: Option<String>,
  pub rejection_actor_user_id: Option<i64>,
  pub events_delivered: bool,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
