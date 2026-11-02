use crate::{domain::*, repository::CollaborationRepository};
use gity_core::AppError;
use gity_entities::project_release_links as entity;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

impl CollaborationRepository {
  pub async fn create_release_link(
    &self,
    command: ReleaseLinkCreate,
  ) -> Result<entity::Model, AppError> {
    let now = chrono::Utc::now().naive_utc();
    Ok(
      entity::ActiveModel {
        id: Set(gity_core::new_id()),
        project_release_id: Set(command.project_release_id),
        name: Set(command.name),
        url: Set(command.url),
        link_type: Set(command.link_type),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&self.connection)
      .await?,
    )
  }
}

impl CollaborationRepository {
  pub async fn release_links(&self, project: i64, id: i64) -> Result<Vec<entity::Model>, AppError> {
    self.release(project, id).await?;
    Ok(
      entity::Entity::find()
        .filter(entity::Column::ProjectReleaseId.eq(id))
        .order_by_asc(entity::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn delete_release_link(
    &self,
    project: i64,
    release: i64,
    id: i64,
  ) -> Result<(), AppError> {
    self.release(project, release).await?;
    let row = entity::Entity::find_by_id(id)
      .filter(entity::Column::ProjectReleaseId.eq(release))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("release link not found"))?;
    entity::Entity::delete_by_id(row.id)
      .exec(&self.connection)
      .await?;
    Ok(())
  }
}
