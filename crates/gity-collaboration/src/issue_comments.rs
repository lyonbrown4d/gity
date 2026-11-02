use crate::{domain::*, repository::CollaborationRepository};
use gity_core::AppError;
use gity_entities::project_issue_comments as entity;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

impl CollaborationRepository {
  pub async fn create_issue_comment(
    &self,
    command: IssueCommentCreate,
  ) -> Result<entity::Model, AppError> {
    let now = chrono::Utc::now().naive_utc();
    Ok(
      entity::ActiveModel {
        id: Set(gity_core::new_id()),
        project_issue_id: Set(command.project_issue_id),
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
  pub async fn issue_comments(
    &self,
    project: i64,
    iid: i64,
  ) -> Result<Vec<entity::Model>, AppError> {
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
