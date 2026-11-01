use super::*;
use serde_json::json;

#[test]
fn audit_counts_distinct_actors_and_nonempty_target_types() {
  let rows = [
    json!({"actor_user_id":"9007199254740993", "target_type":"repository"}),
    json!({"actor_user_id":"9007199254740993", "target_type":"ci"}),
    json!({"actor_user_id":null, "target_type":""}),
    json!({"actor_user_id":0, "target_type":"repository"}),
  ];
  let events = rows.iter().map(Event::from).collect::<Vec<_>>();
  let stats = Stats::from_events(&events);
  assert_eq!((stats.events, stats.actors, stats.target_types), (4, 3, 2));
  assert_eq!(events[0].actor, "9007199254740993");
  assert_eq!(events[2].actor, "system");
  assert_eq!(events[3].actor, "0");
}

#[test]
fn audit_summary_prefers_summary_and_preserves_payload_fallback() {
  assert_eq!(
    Event::from(&json!({"summary":"Created branch","payload":"detail"})).summary,
    "Created branch"
  );
  assert_eq!(
    Event::from(&json!({"summary":"","payload":{"branch":"main"}})).summary,
    r#"{"branch":"main"}"#
  );
  let empty = Event::from(&Value::Null);
  assert!(empty.summary.is_empty());
  assert!(empty.target_id.is_empty());
}
