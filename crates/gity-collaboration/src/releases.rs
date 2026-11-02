use crate::{domain::*, repository::CollaborationRepository};
use gity_core::AppError;
use gity_entities::project_releases as entity;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set,
};

impl CollaborationRepository {
  pub async fn release(&self, project: i64, key: i64) -> Result<entity::Model, AppError> {
    entity::Entity::find()
      .filter(entity::Column::ProjectId.eq(project))
      .filter(entity::Column::Id.eq(key))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("resource not found"))
  }
  pub async fn releases(&self, project: i64) -> Result<Vec<entity::Model>, AppError> {
    Ok(
      entity::Entity::find()
        .filter(entity::Column::ProjectId.eq(project))
        .order_by_desc(entity::Column::ReleasedAt)
        .order_by_desc(entity::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn create_release(&self, command: ReleaseCreate) -> Result<entity::Model, AppError> {
    let now = chrono::Utc::now().naive_utc();
    Ok(
      entity::ActiveModel {
        id: Set(gity_core::new_id()),
        project_id: Set(command.project_id),
        tag_name: Set(command.tag_name),
        name: Set(command.name),
        description: Set(command.description),
        created_by_user_id: Set(command.created_by_user_id),
        released_at: Set(command.released_at),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&self.connection)
      .await?,
    )
  }
  pub async fn update_release(
    &self,
    project: i64,
    key: i64,
    changes: ReleaseChanges,
  ) -> Result<entity::Model, AppError> {
    let current = self.release(project, key).await?;
    let mut active = current.into_active_model();
    if let Change::Set(value) = changes.name {
      active.name = Set(value);
    }
    if let Change::Set(value) = changes.description {
      active.description = Set(value);
    }
    if let Change::Set(value) = changes.released_at {
      active.released_at = Set(value);
    }
    active.updated_at = Set(chrono::Utc::now().naive_utc());
    Ok(active.update(&self.connection).await?)
  }
}

impl CollaborationRepository {
  pub async fn find_release(
    &self,
    project: i64,
    tag: &str,
  ) -> Result<Option<entity::Model>, AppError> {
    Ok(
      entity::Entity::find()
        .filter(entity::Column::ProjectId.eq(project))
        .filter(entity::Column::TagName.eq(tag))
        .one(&self.connection)
        .await?,
    )
  }
  pub async fn delete_release(&self, project: i64, id: i64) -> Result<(), AppError> {
    use sea_orm::TransactionTrait;
    let row = self.release(project, id).await?;
    let tx = self.connection.begin().await?;
    gity_entities::project_release_links::Entity::delete_many()
      .filter(gity_entities::project_release_links::Column::ProjectReleaseId.eq(row.id))
      .exec(&tx)
      .await?;
    entity::Entity::delete_by_id(row.id).exec(&tx).await?;
    tx.commit().await?;
    Ok(())
  }
}
