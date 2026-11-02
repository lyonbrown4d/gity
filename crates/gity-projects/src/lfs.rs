use crate::{LockQuery, RepositoryAccessRepository};
use gity_core::AppError;
use gity_entities::{project_lfs_locks as locks, project_lfs_objects as objects, projects, users};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter,
  QueryOrder, QuerySelect, Set, TransactionTrait, sea_query::Expr,
};
pub(crate) async fn lock_project<C: ConnectionTrait>(db: &C, project: i64) -> Result<(), AppError> {
  projects::Entity::update_many()
    .col_expr(
      projects::Column::UpdatedAt,
      Expr::col(projects::Column::UpdatedAt),
    )
    .filter(projects::Column::Id.eq(project))
    .exec(db)
    .await?;
  projects::Entity::find_by_id(project)
    .one(db)
    .await?
    .ok_or_else(|| AppError::not_found("project not found"))?;
  Ok(())
}
impl RepositoryAccessRepository {
  pub(crate) async fn lfs_object(
    &self,
    project: i64,
    oid: &str,
  ) -> Result<Option<objects::Model>, AppError> {
    Ok(
      objects::Entity::find()
        .filter(objects::Column::ProjectId.eq(project))
        .filter(objects::Column::Oid.eq(oid))
        .one(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn save_lfs_object(
    &self,
    project: i64,
    oid: &str,
    size: i64,
    key: String,
    missing_storage_key: Option<&str>,
  ) -> Result<objects::Model, AppError> {
    let tx = self.connection.begin().await?;
    lock_project(&tx, project).await?;
    let existing = objects::Entity::find()
      .filter(objects::Column::ProjectId.eq(project))
      .filter(objects::Column::Oid.eq(oid))
      .one(&tx)
      .await?;
    let now = chrono::Utc::now().naive_utc();
    let row = if let Some(existing) = existing {
      if missing_storage_key != Some(existing.storage_key.as_str()) {
        tx.commit().await?;
        return Ok(existing);
      }
      let mut active = existing.into_active_model();
      active.byte_size = Set(size);
      active.storage_key = Set(key);
      active.updated_at = Set(now);
      active.update(&tx).await?
    } else {
      objects::ActiveModel {
        id: Set(gity_core::new_id()),
        project_id: Set(project),
        oid: Set(oid.into()),
        byte_size: Set(size),
        storage_key: Set(key),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&tx)
      .await?
    };
    tx.commit().await?;
    Ok(row)
  }
  pub(crate) async fn lfs_objects(
    &self,
    project: i64,
    cursor: i64,
    limit: u64,
  ) -> Result<Vec<objects::Model>, AppError> {
    Ok(
      objects::Entity::find()
        .filter(objects::Column::ProjectId.eq(project))
        .filter(objects::Column::Id.gt(cursor))
        .order_by_asc(objects::Column::Id)
        .limit(limit)
        .all(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn lfs_locks(
    &self,
    project: i64,
    query: LockQuery,
  ) -> Result<Vec<locks::Model>, AppError> {
    let mut select = locks::Entity::find()
      .filter(locks::Column::ProjectId.eq(project))
      .filter(locks::Column::Id.gt(query.cursor));
    if let Some(path) = query.path {
      select = select.filter(locks::Column::Path.eq(path));
    }
    if let Some(id) = query.id {
      select = select.filter(locks::Column::Id.eq(id));
    }
    Ok(
      select
        .order_by_asc(locks::Column::Id)
        .limit(query.limit)
        .all(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn lfs_owner(&self, user: i64) -> Result<users::Model, AppError> {
    users::Entity::find_by_id(user)
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("lock owner not found"))
  }
  pub(crate) async fn create_lfs_lock(
    &self,
    project: i64,
    user: i64,
    path: String,
  ) -> Result<locks::Model, AppError> {
    let now = chrono::Utc::now().naive_utc();
    Ok(
      locks::ActiveModel {
        id: Set(gity_core::new_id()),
        project_id: Set(project),
        owner_user_id: Set(user),
        path: Set(path),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&self.connection)
      .await?,
    )
  }
  pub(crate) async fn delete_lfs_lock(
    &self,
    project: i64,
    user: i64,
    id: i64,
    force: bool,
  ) -> Result<locks::Model, AppError> {
    let tx = self.connection.begin().await?;
    lock_project(&tx, project).await?;
    let row = locks::Entity::find_by_id(id)
      .filter(locks::Column::ProjectId.eq(project))
      .one(&tx)
      .await?
      .ok_or_else(|| AppError::not_found("resource not found"))?;
    if row.owner_user_id != user && !force {
      return Err(AppError::forbidden("lfs lock is owned by another user"));
    }
    locks::Entity::delete_by_id(id).exec(&tx).await?;
    tx.commit().await?;
    Ok(row)
  }
}
