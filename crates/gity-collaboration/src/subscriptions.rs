//! Explicit subscriptions to one issue or merge request.
use crate::{
  domain::{self, SubscriptionCommand, SubscriptionFilter, SubscriptionState},
  repository::CollaborationRepository,
};
use gity_core::AppError;
use gity_entities::project_subscriptions as subscriptions;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, sea_query::OnConflict};

impl CollaborationRepository {
  /// Resolve the project-local `iid` to the stored target before writing.
  async fn target_id(&self, command: &SubscriptionCommand) -> Result<i64, AppError> {
    match domain::target_type(&command.target_type).map_err(AppError::bad_request)? {
      domain::ISSUE_TARGET => Ok(self.issue(command.project, command.iid).await?.id),
      _ => Ok(self.merge_request(command.project, command.iid).await?.id),
    }
  }

  pub async fn subscription_state(
    &self,
    project: i64,
    user: i64,
    target_type: &str,
    target_id: i64,
  ) -> Result<SubscriptionState, AppError> {
    let subscribed = subscriptions::Entity::find()
      .filter(subscriptions::Column::ProjectId.eq(project))
      .filter(subscriptions::Column::UserId.eq(user))
      .filter(subscriptions::Column::TargetType.eq(target_type))
      .filter(subscriptions::Column::TargetId.eq(target_id))
      .one(&self.connection)
      .await?
      .is_some();
    Ok(SubscriptionState {
      project_id: project,
      target_type: target_type.to_owned(),
      target_id,
      subscribed,
    })
  }

  pub async fn subscribe(
    &self,
    command: SubscriptionCommand,
  ) -> Result<SubscriptionState, AppError> {
    let target_type = domain::target_type(&command.target_type).map_err(AppError::bad_request)?;
    let target_id = self.target_id(&command).await?;
    // Repeated subscription requests stay idempotent through the unique index.
    subscriptions::Entity::insert(subscriptions::ActiveModel {
      id: Set(gity_core::new_id()),
      project_id: Set(command.project),
      user_id: Set(command.user),
      target_type: Set(target_type.to_owned()),
      target_id: Set(target_id),
      created_at: Set(chrono::Utc::now().naive_utc()),
    })
    .on_conflict(
      OnConflict::columns([
        subscriptions::Column::UserId,
        subscriptions::Column::TargetType,
        subscriptions::Column::TargetId,
      ])
      .do_nothing_on([subscriptions::Column::Id])
      .to_owned(),
    )
    .try_insert()
    .exec(&self.connection)
    .await?;
    self
      .subscription_state(command.project, command.user, target_type, target_id)
      .await
  }

  pub async fn unsubscribe(
    &self,
    command: SubscriptionCommand,
  ) -> Result<SubscriptionState, AppError> {
    let target_type = domain::target_type(&command.target_type).map_err(AppError::bad_request)?;
    let target_id = self.target_id(&command).await?;
    subscriptions::Entity::delete_many()
      .filter(subscriptions::Column::ProjectId.eq(command.project))
      .filter(subscriptions::Column::UserId.eq(command.user))
      .filter(subscriptions::Column::TargetType.eq(target_type))
      .filter(subscriptions::Column::TargetId.eq(target_id))
      .exec(&self.connection)
      .await?;
    Ok(SubscriptionState {
      project_id: command.project,
      target_type: target_type.to_owned(),
      target_id,
      subscribed: false,
    })
  }

  pub async fn subscription(
    &self,
    command: SubscriptionCommand,
  ) -> Result<SubscriptionState, AppError> {
    let target_type = domain::target_type(&command.target_type).map_err(AppError::bad_request)?;
    let target_id = self.target_id(&command).await?;
    self
      .subscription_state(command.project, command.user, target_type, target_id)
      .await
  }

  pub async fn subscriptions(
    &self,
    user: i64,
    filter: SubscriptionFilter,
  ) -> Result<Vec<subscriptions::Model>, AppError> {
    let mut query = subscriptions::Entity::find().filter(subscriptions::Column::UserId.eq(user));
    if let Some(raw) = filter.target_type.as_deref() {
      let target_type = domain::target_type(raw).map_err(AppError::bad_request)?;
      query = query.filter(subscriptions::Column::TargetType.eq(target_type));
    }
    if let Some(target_id) = filter.target_id {
      if target_id <= 0 {
        return Err(AppError::bad_request("target_id must be positive"));
      }
      query = query.filter(subscriptions::Column::TargetId.eq(target_id));
    }
    Ok(
      query
        .order_by_desc(subscriptions::Column::CreatedAt)
        .order_by_desc(subscriptions::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }

  /// Subscribers of one target receive comment notifications in addition to
  /// the participants notified by the calling feature.
  pub async fn subscribed_user_ids(
    &self,
    project: i64,
    target_type: &str,
    target_id: i64,
  ) -> Result<Vec<i64>, AppError> {
    Ok(
      subscriptions::Entity::find()
        .filter(subscriptions::Column::ProjectId.eq(project))
        .filter(subscriptions::Column::TargetType.eq(target_type))
        .filter(subscriptions::Column::TargetId.eq(target_id))
        .order_by_asc(subscriptions::Column::Id)
        .all(&self.connection)
        .await?
        .into_iter()
        .map(|row| row.user_id)
        .collect(),
    )
  }
}
