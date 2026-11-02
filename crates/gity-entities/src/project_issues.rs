use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "project_issues")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: i64,
  pub project_id: i64,
  pub iid: i64,
  pub author_user_id: i64,
  pub title: String,
  pub description: Option<String>,
  pub state: String,
  pub milestone_id: Option<i64>,
  pub due_date: Option<chrono::NaiveDate>,
  pub weight: Option<i64>,
  pub confidential: i64,
  pub closed_by_user_id: Option<i64>,
  pub closed_at: Option<chrono::NaiveDateTime>,
  pub updated_by_user_id: Option<i64>,
  pub issue_type: String,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
