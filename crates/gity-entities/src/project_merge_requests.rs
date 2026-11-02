use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "project_merge_requests")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub project_id: i64,
  pub iid: i64,
  pub author_user_id: i64,
  pub title: String,
  pub description: Option<String>,
  pub state: String,
  pub source_branch: String,
  pub target_branch: String,
  pub milestone_id: Option<i64>,
  pub merged_at: Option<chrono::NaiveDateTime>,
  pub merged_by_user_id: Option<i64>,
  pub merge_commit_sha: Option<String>,
  pub head_sha: Option<String>,
  pub closed_at: Option<chrono::NaiveDateTime>,
  pub squash: i64,
  pub draft: i64,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
