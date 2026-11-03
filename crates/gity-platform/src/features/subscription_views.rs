use gity_collaboration::domain::{NotificationPreferenceState, SubscriptionState};
use gity_entities::project_subscriptions as subscriptions;
use serde_json::{Value, json};

pub(super) fn subscription_view(row: subscriptions::Model) -> Value {
  json!({
    "project_id": row.project_id,
    "target_type": row.target_type,
    "target_id": row.target_id,
    "created_at": stamp(row.created_at),
  })
}

pub(super) fn state_view(state: SubscriptionState) -> Value {
  json!({
    "project_id": state.project_id,
    "target_type": state.target_type,
    "target_id": state.target_id,
    "subscribed": state.subscribed,
  })
}

pub(super) fn preference_view(state: NotificationPreferenceState) -> Value {
  json!({
    "kind": state.kind,
    "enabled": state.enabled,
    "explicit": state.explicit,
    "created_at": state.created_at.map(stamp),
    "updated_at": state.updated_at.map(stamp),
  })
}

fn stamp(value: chrono::NaiveDateTime) -> String {
  value
    .and_utc()
    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
