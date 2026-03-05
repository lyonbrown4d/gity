use chrono::Utc;
use mr_ulid::Ulid;
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::prelude::async_trait::async_trait;
pub use sea_orm::{ActiveModelBehavior, DeriveRelation, EnumIter, Related, RelationDef};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "repository_language_snapshots")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: String,

  pub repository_id: String,
  pub branch_name: String,
  pub revision: String,
  pub total_bytes: i64,
  pub analyzed_at: DateTimeWithTimeZone,
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
  #[sea_orm(has_many = "super::repository_language_snapshot_items::Entity")]
  Items,
}

impl Related<super::repositories::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Repository.def()
  }
}

impl Related<super::repository_language_snapshot_items::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Items.def()
  }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
  fn new() -> Self {
    let now = Utc::now().into();
    Self {
      id: Set(Ulid::new().to_string()),
      analyzed_at: Set(now),
      created_at: Set(now),
      ..ActiveModelTrait::default()
    }
  }
}
