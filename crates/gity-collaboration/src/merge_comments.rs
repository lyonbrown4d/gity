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
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&self.connection)
      .await?,
    )
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
