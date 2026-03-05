use chrono::Utc;
use mr_ulid::Ulid;
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::prelude::async_trait::async_trait;
pub use sea_orm::{
  ActiveModelBehavior, DeriveActiveEnum, DeriveRelation, EnumIter, Related, RelationDef,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "repository_issues")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: String,

  pub repository_id: String,
  pub number: i32,
  pub title: String,
  pub description: Option<String>,
  pub status: RepositoryIssueStatus,
  pub author_user_id: String,
  pub assignee_user_id: Option<String>,
  pub created_at: DateTimeWithTimeZone,
  pub updated_at: DateTimeWithTimeZone,
  pub closed_at: Option<DateTimeWithTimeZone>,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum RepositoryIssueStatus {
  #[sea_orm(string_value = "open")]
  Open,
  #[sea_orm(string_value = "closed")]
  Closed,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::repositories::Entity",
    from = "Column::RepositoryId",
    to = "super::repositories::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  Repository,
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::AuthorUserId",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  Author,
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::AssigneeUserId",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "SetNull"
  )]
  Assignee,
  #[sea_orm(has_many = "super::repository_issue_comments::Entity")]
  Comments,
}

impl Related<super::repositories::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Repository.def()
  }
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Author.def()
  }
}

impl Related<super::repository_issue_comments::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Comments.def()
  }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
  fn new() -> Self {
    let now = Utc::now().into();
    Self {
      id: Set(Ulid::new().to_string()),
      status: Set(RepositoryIssueStatus::Open),
      created_at: Set(now),
      updated_at: Set(now),
      ..ActiveModelTrait::default()
    }
  }
}
