use crate::{domain::*, repository::CollaborationRepository};
use gity_core::AppError;
use gity_entities::project_issue_attachments as entity;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

impl CollaborationRepository {
  pub async fn create_attachment(
    &self,
    command: AttachmentCreate,
  ) -> Result<entity::Model, AppError> {
    let now = chrono::Utc::now().naive_utc();
    Ok(
      entity::ActiveModel {
        id: Set(if command.id > 0 {
          command.id
        } else {
          gity_core::new_id()
        }),
        project_issue_id: Set(command.project_issue_id),
        uploaded_by_user_id: Set(command.uploaded_by_user_id),
        file_name: Set(command.file_name),
        content_type: Set(command.content_type),
        byte_size: Set(command.byte_size),
        storage_key: Set(command.storage_key),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&self.connection)
      .await?,
    )
  }
}

impl CollaborationRepository {
  pub async fn attachments(&self, project: i64, iid: i64) -> Result<Vec<entity::Model>, AppError> {
    let parent = self.issue(project, iid).await?;
    Ok(
      entity::Entity::find()
        .filter(entity::Column::ProjectIssueId.eq(parent.id))
        .order_by_asc(entity::Column::CreatedAt)
        .order_by_asc(entity::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
}

impl CollaborationRepository {
  pub async fn attachment(
    &self,
    project: i64,
    iid: i64,
    id: i64,
  ) -> Result<entity::Model, AppError> {
    let issue = self.issue(project, iid).await?;
    entity::Entity::find_by_id(id)
      .filter(entity::Column::ProjectIssueId.eq(issue.id))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("attachment not found"))
  }
}
