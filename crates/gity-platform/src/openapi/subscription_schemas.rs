//! Subscription and notification-preference request and response schemas.
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Subscription {
  pub project_id: String,
  /// `issue` or `merge_request`.
  #[schema(example = "issue")]
  pub target_type: String,
  /// Application-generated issue or merge request identifier.
  pub target_id: String,
  #[schema(format=DateTime)]
  pub created_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct SubscriptionState {
  pub project_id: String,
  #[schema(example = "issue")]
  pub target_type: String,
  pub target_id: String,
  pub subscribed: bool,
}

#[derive(Serialize, ToSchema)]
pub struct NotificationPreference {
  #[schema(example = "issue_commented")]
  pub kind: String,
  pub enabled: bool,
  /// True when the user saved this preference; unlisted kinds default to enabled.
  pub explicit: bool,
  #[schema(format=DateTime)]
  pub created_at: Option<String>,
  #[schema(format=DateTime)]
  pub updated_at: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateNotificationPreference {
  #[schema(example = "issue_commented")]
  pub kind: String,
  pub enabled: bool,
}
