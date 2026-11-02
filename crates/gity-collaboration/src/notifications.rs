//! Todo notifications filtered by explicit user preferences and subscriptions.
use crate::{
  domain::{self, Notification, NotificationPreferenceChange, NotificationSubject},
  repository::CollaborationRepository,
};
use gity_core::AppError;
use gity_entities::{
  project_issue_comments as issue_comments, project_merge_request_comments as merge_comments,
  project_todos as todos, projects, user_notification_preferences as preferences,
};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
  sea_query::OnConflict,
};

impl CollaborationRepository {
  pub async fn todos(
    &self,
    user: i64,
    state: &str,
    limit: u64,
  ) -> Result<Vec<todos::Model>, AppError> {
    let mut query = todos::Entity::find().filter(todos::Column::UserId.eq(user));
    if !state.is_empty() {
      query = query.filter(todos::Column::State.eq(state));
    }
    Ok(
      query
        .order_by_desc(todos::Column::CreatedAt)
        .order_by_desc(todos::Column::Id)
        .limit(limit)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn complete_todo(&self, user: i64, id: i64) -> Result<(), AppError> {
    let row = todos::Entity::find_by_id(id)
      .filter(todos::Column::UserId.eq(user))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("todo not found"))?;
    let now = chrono::Utc::now().naive_utc();
    todos::ActiveModel {
      id: Set(row.id),
      state: Set("done".to_owned()),
      done_at: Set(Some(now)),
      updated_at: Set(now),
      ..Default::default()
    }
    .update(&self.connection)
    .await?;
    Ok(())
  }

  /// Notify participants and subscribers. A muted kind or the acting comment
  /// author is skipped; targets without a subscription subject keep the
  /// previous participant-only behaviour.
  pub async fn notify(&self, command: Notification) -> Result<(), AppError> {
    let project = projects::Entity::find_by_id(command.project)
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("project not found"))?;
    let mut recipients = vec![command.user];
    let mut actor = None;
    if let Some(subject) = self.notification_subject(&command).await? {
      // Only current project or organization members receive the extra copy.
      let members = self.recipients(command.project).await?;
      recipients.extend(
        self
          .subscribed_user_ids(command.project, &subject.target_type, subject.target_id)
          .await?
          .into_iter()
          .filter(|user| members.contains(user)),
      );
      actor = Some(subject.actor);
    }
    recipients.sort_unstable();
    recipients.dedup();
    for user in recipients {
      if actor == Some(user) || !self.notification_enabled(user, &command.kind).await? {
        continue;
      }
      self.insert_todo(user, &project, &command).await?;
    }
    Ok(())
  }

  async fn insert_todo(
    &self,
    user: i64,
    project: &projects::Model,
    command: &Notification,
  ) -> Result<(), AppError> {
    let now = chrono::Utc::now().naive_utc();
    let model = todos::ActiveModel {
      id: Set(gity_core::new_id()),
      user_id: Set(user),
      organization_id: Set(project.organization_id),
      project_id: Set(command.project),
      kind: Set(command.kind.clone()),
      state: Set("pending".to_owned()),
      target_type: Set(command.target_type.clone()),
      target_id: Set(command.target.clone()),
      title: Set(command.title.clone()),
      summary: Set(command.summary.clone()),
      action_url: Set(format!(
        "/app/projects/{}/{}{}",
        project.organization_id, project.id, command.suffix
      )),
      created_at: Set(now),
      updated_at: Set(now),
      done_at: Set(None),
    };
    todos::Entity::insert(model)
      .on_conflict(
        OnConflict::columns([
          todos::Column::UserId,
          todos::Column::Kind,
          todos::Column::TargetType,
          todos::Column::TargetId,
        ])
        .do_nothing_on([todos::Column::Id])
        .to_owned(),
      )
      .try_insert()
      .exec(&self.connection)
      .await?;
    Ok(())
  }

  /// Comment notifications resolve their issue or merge request and its author,
  /// which keeps authors from receiving their own comment notification.
  async fn notification_subject(
    &self,
    command: &Notification,
  ) -> Result<Option<NotificationSubject>, AppError> {
    let Ok(target) = command.target.parse::<i64>() else {
      return Ok(None);
    };
    match command.target_type.as_str() {
      "issue_comment" => {
        let Some(row) = issue_comments::Entity::find_by_id(target)
          .one(&self.connection)
          .await?
        else {
          return Ok(None);
        };
        Ok(Some(NotificationSubject {
          target_type: domain::ISSUE_TARGET.to_owned(),
          target_id: row.project_issue_id,
          actor: row.author_user_id,
        }))
      }
      "merge_request_comment" => {
        let Some(row) = merge_comments::Entity::find_by_id(target)
          .one(&self.connection)
          .await?
        else {
          return Ok(None);
        };
        Ok(Some(NotificationSubject {
          target_type: domain::MERGE_REQUEST_TARGET.to_owned(),
          target_id: row.merge_request_id,
          actor: row.author_user_id,
        }))
      }
      _ => Ok(None),
    }
  }

  /// Only an explicit disabled preference mutes a kind.
  pub async fn notification_enabled(&self, user: i64, kind: &str) -> Result<bool, AppError> {
    Ok(
      preferences::Entity::find()
        .filter(preferences::Column::UserId.eq(user))
        .filter(preferences::Column::Kind.eq(kind))
        .one(&self.connection)
        .await?
        .map(|row| row.enabled)
        .unwrap_or(true),
    )
  }

  pub async fn notification_preferences(
    &self,
    user: i64,
  ) -> Result<Vec<preferences::Model>, AppError> {
    Ok(
      preferences::Entity::find()
        .filter(preferences::Column::UserId.eq(user))
        .order_by_asc(preferences::Column::Kind)
        .all(&self.connection)
        .await?,
    )
  }

  pub async fn set_notification_preference(
    &self,
    user: i64,
    change: NotificationPreferenceChange,
  ) -> Result<preferences::Model, AppError> {
    if !domain::valid_kind(&change.kind) {
      return Err(AppError::bad_request("invalid notification kind"));
    }
    let kind = change.kind.trim().to_owned();
    let now = chrono::Utc::now().naive_utc();
    let existing = preferences::Entity::find()
      .filter(preferences::Column::UserId.eq(user))
      .filter(preferences::Column::Kind.eq(kind.clone()))
      .one(&self.connection)
      .await?;
    if let Some(row) = existing {
      return Ok(
        preferences::ActiveModel {
          id: Set(row.id),
          enabled: Set(change.enabled),
          updated_at: Set(now),
          ..Default::default()
        }
        .update(&self.connection)
        .await?,
      );
    }
    Ok(
      preferences::ActiveModel {
        id: Set(gity_core::new_id()),
        user_id: Set(user),
        kind: Set(kind),
        enabled: Set(change.enabled),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&self.connection)
      .await?,
    )
  }
}
