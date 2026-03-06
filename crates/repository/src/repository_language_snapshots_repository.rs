use crate::BaseRepository;
use entity::{repository_language_snapshot_items, repository_language_snapshots};
use sea_orm::{
  ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
};

#[derive(Clone)]
pub struct RepositoryLanguageSnapshotsRepository<C: ConnectionTrait = DatabaseConnection> {
  base: BaseRepository<C>,
}

impl<C: ConnectionTrait> RepositoryLanguageSnapshotsRepository<C> {
  pub fn new(conn: C) -> Self {
    Self {
      base: BaseRepository::new(conn),
    }
  }

  pub fn connection(&self) -> &C {
    self.base.connection()
  }

  pub async fn insert_snapshot(
    &self,
    active: repository_language_snapshots::ActiveModel,
  ) -> Result<repository_language_snapshots::Model, DbErr> {
    self.base.insert(active).await
  }

  pub async fn insert_snapshot_item(
    &self,
    active: repository_language_snapshot_items::ActiveModel,
  ) -> Result<repository_language_snapshot_items::Model, DbErr> {
    self.base.insert(active).await
  }

  pub async fn find_latest_snapshot_by_repo_and_branch(
    &self,
    repository_id: &str,
    branch_name: &str,
  ) -> Result<Option<repository_language_snapshots::Model>, DbErr> {
    repository_language_snapshots::Entity::find()
      .filter(repository_language_snapshots::Column::RepositoryId.eq(repository_id.to_string()))
      .filter(repository_language_snapshots::Column::BranchName.eq(branch_name.to_string()))
      .order_by_desc(repository_language_snapshots::Column::AnalyzedAt)
      .order_by_desc(repository_language_snapshots::Column::CreatedAt)
      .one(self.connection())
      .await
  }

  pub async fn list_snapshot_items_by_snapshot_id(
    &self,
    snapshot_id: &str,
  ) -> Result<Vec<repository_language_snapshot_items::Model>, DbErr> {
    repository_language_snapshot_items::Entity::find()
      .filter(repository_language_snapshot_items::Column::SnapshotId.eq(snapshot_id.to_string()))
      .order_by_desc(repository_language_snapshot_items::Column::Bytes)
      .order_by_asc(repository_language_snapshot_items::Column::Language)
      .all(self.connection())
      .await
  }
}
