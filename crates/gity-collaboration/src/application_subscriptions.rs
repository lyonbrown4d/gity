//! Subscription and notification-preference use cases exposed to adapters.
use crate::{
  CollaborationService,
  domain::{
    self, NotificationPreferenceChange, NotificationPreferenceState, SubscriptionCommand,
    SubscriptionFilter, SubscriptionState,
  },
};
use gity_core::AppError;
use gity_entities::project_subscriptions as subscriptions;

impl CollaborationService {
  pub async fn subscribe(
    &self,
    command: SubscriptionCommand,
  ) -> Result<SubscriptionState, AppError> {
    self.repository.subscribe(command).await
  }
  pub async fn unsubscribe(
    &self,
    command: SubscriptionCommand,
  ) -> Result<SubscriptionState, AppError> {
    self.repository.unsubscribe(command).await
  }
  pub async fn subscription(
    &self,
    command: SubscriptionCommand,
  ) -> Result<SubscriptionState, AppError> {
    self.repository.subscription(command).await
  }
  pub async fn subscriptions(
    &self,
    user: i64,
    filter: SubscriptionFilter,
  ) -> Result<Vec<subscriptions::Model>, AppError> {
    self.repository.subscriptions(user, filter).await
  }
  /// Persisted rows override the enabled default of every known kind.
  pub async fn notification_preferences(
    &self,
    user: i64,
  ) -> Result<Vec<NotificationPreferenceState>, AppError> {
    let rows = self.repository.notification_preferences(user).await?;
    let mut states = domain::NOTIFICATION_KINDS
      .iter()
      .map(|kind| NotificationPreferenceState {
        kind: (*kind).to_owned(),
        enabled: true,
        explicit: false,
        created_at: None,
        updated_at: None,
      })
      .collect::<Vec<_>>();
    for row in rows {
      let state = NotificationPreferenceState {
        kind: row.kind.clone(),
        enabled: row.enabled,
        explicit: true,
        created_at: Some(row.created_at),
        updated_at: Some(row.updated_at),
      };
      match states.iter_mut().find(|item| item.kind == state.kind) {
        Some(item) => *item = state,
        None => states.push(state),
      }
    }
    states.sort_by(|left, right| left.kind.cmp(&right.kind));
    Ok(states)
  }
  pub async fn set_notification_preference(
    &self,
    user: i64,
    change: NotificationPreferenceChange,
  ) -> Result<NotificationPreferenceState, AppError> {
    let row = self
      .repository
      .set_notification_preference(user, change)
      .await?;
    Ok(NotificationPreferenceState {
      kind: row.kind,
      enabled: row.enabled,
      explicit: true,
      created_at: Some(row.created_at),
      updated_at: Some(row.updated_at),
    })
  }
}
