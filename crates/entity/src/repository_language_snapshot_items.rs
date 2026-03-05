use chrono::Utc;
use mr_ulid::Ulid;
use sea_orm::Set;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::prelude::async_trait::async_trait;
pub use sea_orm::{ActiveModelBehavior, DeriveRelation, EnumIter, Related, RelationDef};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "repository_language_snapshot_items")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: String,

  pub snapshot_id: String,
  pub language: String,
  pub bytes: i64,
  pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::repository_language_snapshots::Entity",
    from = "Column::SnapshotId",
    to = "super::repository_language_snapshots::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  Snapshot,
}

impl Related<super::repository_language_snapshots::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Snapshot.def()
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
