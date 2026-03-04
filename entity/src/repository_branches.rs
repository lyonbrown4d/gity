use chrono::Utc;
use mr_ulid::Ulid;
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::prelude::async_trait::async_trait;
pub use sea_orm::{ActiveModelBehavior, DeriveRelation, EnumIter, Related, RelationDef};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "repository_branches")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: String,

  pub repository_id: String,
  pub name: String,
  pub is_protected: bool,
  pub last_commit_sha: Option<String>,

  pub created_at: DateTimeWithTimeZone,
  pub updated_at: DateTimeWithTimeZone,
  pub deleted_at: Option<DateTimeWithTimeZone>,
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
}

impl Related<super::repositories::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Repository.def()
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
