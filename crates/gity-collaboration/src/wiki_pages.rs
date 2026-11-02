use crate::{domain::*, repository::CollaborationRepository};
use gity_core::AppError;
use gity_entities::project_wiki_pages as entity;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set,
};

impl CollaborationRepository {
  pub async fn create_wiki_page(&self, command: WikiPageCreate) -> Result<entity::Model, AppError> {
    let now = chrono::Utc::now().naive_utc();
    Ok(
      entity::ActiveModel {
        id: Set(gity_core::new_id()),
        project_id: Set(command.project_id),
        slug: Set(command.slug),
        title: Set(command.title),
        content: Set(command.content),
        format: Set(command.format),
        author_user_id: Set(command.author_user_id),
        last_edited_by_user_id: Set(command.last_edited_by_user_id),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&self.connection)
      .await?,
    )
  }
  pub async fn update_wiki_page(
    &self,
    project: i64,
    key: &str,
    changes: WikiPageChanges,
  ) -> Result<entity::Model, AppError> {
    let current = self.wiki_page(project, key).await?;
    let mut active = current.into_active_model();
    if let Change::Set(value) = changes.title {
      active.title = Set(value);
    }
    if let Change::Set(value) = changes.content {
      active.content = Set(value);
    }
    if let Change::Set(value) = changes.last_edited_by_user_id {
      active.last_edited_by_user_id = Set(value);
    }
    active.updated_at = Set(chrono::Utc::now().naive_utc());
    Ok(active.update(&self.connection).await?)
  }
}

impl CollaborationRepository {
  pub async fn wiki_page(&self, project: i64, slug: &str) -> Result<entity::Model, AppError> {
    self
      .find_wiki_page(project, slug)
      .await?
      .ok_or_else(|| AppError::not_found("wiki page not found"))
  }
  pub async fn find_wiki_page(
    &self,
    project: i64,
    slug: &str,
  ) -> Result<Option<entity::Model>, AppError> {
    Ok(
      entity::Entity::find()
        .filter(entity::Column::ProjectId.eq(project))
        .filter(entity::Column::Slug.eq(slug))
        .one(&self.connection)
        .await?,
    )
  }
  pub async fn wiki_pages(&self, project: i64) -> Result<Vec<entity::Model>, AppError> {
    Ok(
      entity::Entity::find()
        .filter(entity::Column::ProjectId.eq(project))
        .order_by_asc(entity::Column::Title)
        .order_by_asc(entity::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn delete_wiki_page(
    &self,
    project: i64,
    slug: &str,
  ) -> Result<entity::Model, AppError> {
    let row = self.wiki_page(project, slug).await?;
    entity::Entity::delete_by_id(row.id)
      .exec(&self.connection)
      .await?;
    Ok(row)
  }
}
