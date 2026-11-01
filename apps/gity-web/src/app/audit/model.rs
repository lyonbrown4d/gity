use crate::api;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, PartialEq)]
pub(super) struct Event {
  pub name: String,
  pub action: String,
  pub actor: String,
  pub target_type: String,
  pub target_id: String,
  pub summary: String,
  pub created_at: String,
}
impl From<&Value> for Event {
  fn from(value: &Value) -> Self {
    let actor = api::text(value, "actor_user_id");
    let summary = api::text(value, "summary");
    Self {
      name: api::text(value, "event_name"),
      action: api::text(value, "action"),
      actor: if actor.is_empty() {
        "system".into()
      } else {
        actor
      },
      target_type: api::text(value, "target_type"),
      target_id: api::text(value, "target_id"),
      summary: if summary.is_empty() {
        api::text(value, "payload")
      } else {
        summary
      },
      created_at: api::text(value, "created_at"),
    }
  }
}
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Stats {
  pub events: usize,
  pub actors: usize,
  pub target_types: usize,
}
impl Stats {
  pub fn from_events(events: &[Event]) -> Self {
    Self {
      events: events.len(),
      actors: events
        .iter()
        .map(|event| &event.actor)
        .collect::<BTreeSet<_>>()
        .len(),
      target_types: events
        .iter()
        .map(|event| &event.target_type)
        .filter(|kind| !kind.is_empty())
        .collect::<BTreeSet<_>>()
        .len(),
    }
  }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
