use entity::{repository_language_snapshot_items, repository_language_snapshots};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, QueryOrder,
};

pub struct RepositoryLanguageSnapshotsRepository;

impl RepositoryLanguageSnapshotsRepository {
  pub async fn insert_snapshot<C: ConnectionTrait>(
    conn: &C,
    active: repository_language_snapshots::ActiveModel,
  ) -> Result<repository_language_snapshots::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn insert_snapshot_item<C: ConnectionTrait>(
    conn: &C,
    active: repository_language_snapshot_items::ActiveModel,
  ) -> Result<repository_language_snapshot_items::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn find_latest_snapshot_by_repo_and_branch<C: ConnectionTrait>(
    conn: &C,
    repository_id: &str,
    branch_name: &str,
  ) -> Result<Option<repository_language_snapshots::Model>, DbErr> {
    repository_language_snapshots::Entity::find()
      .filter(repository_language_snapshots::Column::RepositoryId.eq(repository_id.to_string()))
      .filter(repository_language_snapshots::Column::BranchName.eq(branch_name.to_string()))
      .order_by_desc(repository_language_snapshots::Column::AnalyzedAt)
      .order_by_desc(repository_language_snapshots::Column::CreatedAt)
      .one(conn)
      .await
  }

  pub async fn list_snapshot_items_by_snapshot_id<C: ConnectionTrait>(
    conn: &C,
    snapshot_id: &str,
  ) -> Result<Vec<repository_language_snapshot_items::Model>, DbErr> {
    repository_language_snapshot_items::Entity::find()
      .filter(repository_language_snapshot_items::Column::SnapshotId.eq(snapshot_id.to_string()))
      .order_by_desc(repository_language_snapshot_items::Column::Bytes)
      .order_by_asc(repository_language_snapshot_items::Column::Language)
      .all(conn)
      .await
  }
}
