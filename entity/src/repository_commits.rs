use chrono::Utc;
use mr_ulid::Ulid;
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::prelude::async_trait::async_trait;
pub use sea_orm::{ActiveModelBehavior, DeriveRelation, EnumIter, Related, RelationDef};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "repository_commits")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: String,

  pub repository_id: String,
  pub branch_name: String,
  pub commit_sha: String,
  pub message: String,
  pub author_user_id: String,

  pub created_at: DateTimeWithTimeZone,
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

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
  fn new() -> Self {
    Self {
      id: Set(Ulid::new().to_string()),
      created_at: Set(Utc::now().into()),
      ..ActiveModelTrait::default()
    }
  }
}
