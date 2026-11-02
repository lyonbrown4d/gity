use crate::{domain::*, repository::CollaborationRepository};
use gity_core::AppError;
use gity_entities::project_issues as entity;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set,
};

impl CollaborationRepository {
  pub async fn issue(&self, project: i64, key: i64) -> Result<entity::Model, AppError> {
    entity::Entity::find()
      .filter(entity::Column::ProjectId.eq(project))
      .filter(entity::Column::Iid.eq(key))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("resource not found"))
  }
  pub async fn issues(&self, project: i64) -> Result<Vec<entity::Model>, AppError> {
    Ok(
      entity::Entity::find()
        .filter(entity::Column::ProjectId.eq(project))
        .order_by_desc(entity::Column::Iid)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn create_issue(&self, command: IssueCreate) -> Result<entity::Model, AppError> {
    let iid = gity_persistence::next_iid(
      &self.connection,
      command.project_id,
      gity_persistence::CounterKind::Issue,
    )
    .await?;
    let now = chrono::Utc::now().naive_utc();
    Ok(
      entity::ActiveModel {
        id: Set(gity_core::new_id()),
        project_id: Set(command.project_id),
        author_user_id: Set(command.author_user_id),
        title: Set(command.title),
        description: Set(command.description),
        milestone_id: Set(command.milestone_id),
        due_date: Set(command.due_date),
        weight: Set(command.weight),
        confidential: Set(command.confidential),
        issue_type: Set(command.issue_type),
        iid: Set(iid),
        state: Set("opened".to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      }
      .insert(&self.connection)
      .await?,
    )
  }
  pub async fn update_issue(
    &self,
    project: i64,
    key: i64,
    changes: IssueChanges,
  ) -> Result<entity::Model, AppError> {
    let current = self.issue(project, key).await?;
    let mut active = current.into_active_model();
    if let Change::Set(value) = changes.title {
      active.title = Set(value);
    }
    if let Change::Set(value) = changes.description {
      active.description = Set(value);
    }
    if let Change::Set(value) = changes.state {
      active.state = Set(value);
    }
    if let Change::Set(value) = changes.milestone_id {
      active.milestone_id = Set(value);
    }
    if let Change::Set(value) = changes.due_date {
      active.due_date = Set(value);
    }
    if let Change::Set(value) = changes.weight {
      active.weight = Set(value);
    }
    if let Change::Set(value) = changes.confidential {
      active.confidential = Set(value);
    }
    if let Change::Set(value) = changes.issue_type {
      active.issue_type = Set(value);
    }
    if let Change::Set(value) = changes.updated_by_user_id {
      active.updated_by_user_id = Set(value);
    }
    if let Change::Set(value) = changes.closed_at {
      active.closed_at = Set(value);
    }
    if let Change::Set(value) = changes.closed_by_user_id {
      active.closed_by_user_id = Set(value);
    }
    active.updated_at = Set(chrono::Utc::now().naive_utc());
    Ok(active.update(&self.connection).await?)
  }
}
