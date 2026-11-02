use crate::{domain::*, repository::CollaborationRepository};
use gity_core::AppError;
use gity_entities::project_merge_request_comments as entity;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

impl CollaborationRepository {
  pub async fn create_merge_comment(
    &self,
    command: MergeCommentCreate,
  ) -> Result<entity::Model, AppError> {
    let now = chrono::Utc::now().naive_utc();
    Ok(
      entity::ActiveModel {
        id: Set(gity_core::new_id()),
        merge_request_id: Set(command.merge_request_id),
        author_user_id: Set(command.author_user_id),
        body: Set(command.body),
        discussion_id: Set(command.discussion_id),
        position: Set(command.position),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&self.connection)
      .await?,
    )
  }
}

impl CollaborationRepository {
  pub async fn merge_comment(
    &self,
    project: i64,
    iid: i64,
    comment: i64,
  ) -> Result<entity::Model, AppError> {
    let parent = self.merge_request(project, iid).await?;
    entity::Entity::find_by_id(comment)
      .filter(entity::Column::MergeRequestId.eq(parent.id))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("comment not found"))
  }
  pub async fn update_merge_comment(
    &self,
    project: i64,
    iid: i64,
    comment: i64,
    command: MergeCommentUpdate,
  ) -> Result<entity::Model, AppError> {
    let current = self.merge_comment(project, iid, comment).await?;
    let row = entity::ActiveModel {
      id: Set(current.id),
      body: Set(command.body),
      updated_at: Set(chrono::Utc::now().naive_utc()),
      ..Default::default()
    }
    .update(&self.connection)
    .await?;
    Ok(row)
  }
  pub async fn delete_merge_comment(
    &self,
    project: i64,
    iid: i64,
    comment: i64,
  ) -> Result<(), AppError> {
    let current = self.merge_comment(project, iid, comment).await?;
    entity::Entity::delete_by_id(current.id)
      .exec(&self.connection)
      .await?;
    Ok(())
  }
}

impl CollaborationRepository {
  pub async fn merge_comments(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<Vec<entity::Model>, AppError> {
    let parent = self.merge_request(project, iid).await?;
    Ok(
      entity::Entity::find()
        .filter(entity::Column::MergeRequestId.eq(parent.id))
        .order_by_asc(entity::Column::CreatedAt)
        .order_by_asc(entity::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
}
