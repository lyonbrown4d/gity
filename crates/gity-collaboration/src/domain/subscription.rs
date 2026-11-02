//! Subscription and notification-preference commands independent of transport.
use serde::Deserialize;

/// Subscription target kinds shared by storage, API validation and fan-out.
pub const ISSUE_TARGET: &str = "issue";
pub const MERGE_REQUEST_TARGET: &str = "merge_request";

/// Notification kinds emitted today; collaborators add kinds without a schema change.
pub const NOTIFICATION_KINDS: [&str; 9] = [
  "branch_deleted",
  "branch_protection_changed",
  "issue_assigned",
  "issue_commented",
  "issue_created",
  "merge_request_commented",
  "merge_request_created",
  "merge_request_merged",
  "project_created",
];

/// One subscription operation resolved from a project-local `iid`.
#[derive(Clone, Debug)]
pub struct SubscriptionCommand {
  pub project: i64,
  pub user: i64,
  pub target_type: String,
  pub iid: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct SubscriptionFilter {
  pub target_type: Option<String>,
  pub target_id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct SubscriptionState {
  pub project_id: i64,
  pub target_type: String,
  pub target_id: i64,
  pub subscribed: bool,
}

#[derive(Clone, Debug)]
pub struct NotificationPreferenceChange {
  pub kind: String,
  pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct NotificationPreferenceState {
  pub kind: String,
  pub enabled: bool,
  pub explicit: bool,
  pub created_at: Option<chrono::NaiveDateTime>,
  pub updated_at: Option<chrono::NaiveDateTime>,
}

/// Notification subject resolved from a stored target, used to fan out to subscribers.
#[derive(Clone, Debug, PartialEq)]
pub struct NotificationSubject {
  pub target_type: String,
  pub target_id: i64,
  pub actor: i64,
}

/// Normalize a public target type so storage and queries agree on one spelling.
pub fn target_type(value: &str) -> Result<&'static str, &'static str> {
  match value.trim() {
    ISSUE_TARGET => Ok(ISSUE_TARGET),
    MERGE_REQUEST_TARGET => Ok(MERGE_REQUEST_TARGET),
    _ => Err("target_type must be issue or merge_request"),
  }
}

/// Notification kinds are lowercase codes bounded by the stored column width.
pub fn valid_kind(value: &str) -> bool {
  let value = value.trim();
  !value.is_empty()
    && value.len() <= 64
    && value.starts_with(|c: char| c.is_ascii_lowercase())
    && value
      .chars()
      .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}
