//! Discussion threads and the comments that belong to them.
//!
//! A thread is scoped to one project and one issue or merge request; every
//! comment carries the thread it was posted into, so listing a thread never
//! needs to scan by author or timestamp.
use crate::{domain::*, repository::CollaborationRepository};
use gity_core::AppError;
use gity_entities::{
  project_discussions as entity, project_issue_comments, project_merge_request_comments,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

impl CollaborationRepository {
  pub async fn create_discussion(
    &self,
    command: DiscussionCreate,
  ) -> Result<entity::Model, AppError> {
    let now = chrono::Utc::now().naive_utc();
    Ok(
      entity::ActiveModel {
        id: Set(gity_core::new_id()),
        project_id: Set(command.project_id),
        target_type: Set(command.target_type),
        target_id: Set(command.target_id),
        resolved: Set(false),
        resolved_by_user_id: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&self.connection)
      .await?,
    )
  }

  /// One thread, only when it belongs to this project and target.
  pub async fn discussion(
    &self,
    project: i64,
    target: DiscussionTarget,
    target_id: i64,
    discussion: i64,
  ) -> Result<entity::Model, AppError> {
    entity::Entity::find_by_id(discussion)
      .filter(entity::Column::ProjectId.eq(project))
      .filter(entity::Column::TargetType.eq(target.as_str()))
      .filter(entity::Column::TargetId.eq(target_id))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("discussion not found"))
  }

  /// Threads of one issue or merge request, oldest first.
  pub async fn discussions(
    &self,
    project: i64,
    target_type: &str,
    target_id: i64,
  ) -> Result<Vec<entity::Model>, AppError> {
    Ok(
      entity::Entity::find()
        .filter(entity::Column::ProjectId.eq(project))
        .filter(entity::Column::TargetType.eq(target_type))
        .filter(entity::Column::TargetId.eq(target_id))
        .order_by_asc(entity::Column::CreatedAt)
        .order_by_asc(entity::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }

  /// Stores the resolved marker; callers have already authorized the actor.
  pub async fn set_discussion_resolved(
    &self,
    discussion: i64,
    resolved: bool,
    actor: i64,
  ) -> Result<entity::Model, AppError> {
    let current = entity::Entity::find_by_id(discussion)
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("discussion not found"))?;
    Ok(
      entity::ActiveModel {
        id: Set(current.id),
        resolved: Set(resolved),
        resolved_by_user_id: Set(resolved.then_some(actor)),
        updated_at: Set(chrono::Utc::now().naive_utc()),
        ..Default::default()
      }
      .update(&self.connection)
      .await?,
    )
  }

  pub async fn issue_discussion_comments(
    &self,
    discussion: i64,
  ) -> Result<Vec<project_issue_comments::Model>, AppError> {
    Ok(
      project_issue_comments::Entity::find()
        .filter(project_issue_comments::Column::DiscussionId.eq(discussion))
        .order_by_asc(project_issue_comments::Column::CreatedAt)
        .order_by_asc(project_issue_comments::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }

  pub async fn merge_discussion_comments(
    &self,
    discussion: i64,
  ) -> Result<Vec<project_merge_request_comments::Model>, AppError> {
    Ok(
      project_merge_request_comments::Entity::find()
        .filter(project_merge_request_comments::Column::DiscussionId.eq(discussion))
        .order_by_asc(project_merge_request_comments::Column::CreatedAt)
        .order_by_asc(project_merge_request_comments::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }

  /// Author of the first comment, which opened the thread.
  pub async fn discussion_author(
    &self,
    target: DiscussionTarget,
    discussion: i64,
  ) -> Result<Option<i64>, AppError> {
    Ok(match target {
      DiscussionTarget::Issue => project_issue_comments::Entity::find()
        .filter(project_issue_comments::Column::DiscussionId.eq(discussion))
        .order_by_asc(project_issue_comments::Column::CreatedAt)
        .order_by_asc(project_issue_comments::Column::Id)
        .one(&self.connection)
        .await?
        .map(|row| row.author_user_id),
      DiscussionTarget::MergeRequest => project_merge_request_comments::Entity::find()
        .filter(project_merge_request_comments::Column::DiscussionId.eq(discussion))
        .order_by_asc(project_merge_request_comments::Column::CreatedAt)
        .order_by_asc(project_merge_request_comments::Column::Id)
        .one(&self.connection)
        .await?
        .map(|row| row.author_user_id),
    })
  }
}
