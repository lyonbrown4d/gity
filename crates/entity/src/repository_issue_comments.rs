use chrono::Utc;
use mr_ulid::Ulid;
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::prelude::async_trait::async_trait;
pub use sea_orm::{ActiveModelBehavior, DeriveRelation, EnumIter, Related, RelationDef};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "repository_issue_comments")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: String,

  pub issue_id: String,
  pub author_user_id: String,
  pub content: String,
  pub created_at: DateTimeWithTimeZone,
  pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::repository_issues::Entity",
    from = "Column::IssueId",
    to = "super::repository_issues::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  Issue,
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::AuthorUserId",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  Author,
}

impl Related<super::repository_issues::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Issue.def()
  }
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Author.def()
  }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
  fn new() -> Self {
    let now = Utc::now().into();
    Self {
      id: Set(Ulid::new().to_string()),
      created_at: Set(now),
      updated_at: Set(now),
      ..ActiveModelTrait::default()
    }
  }
}
