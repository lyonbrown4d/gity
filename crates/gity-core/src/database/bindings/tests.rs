use super::*;
#[test]
fn converts_schema_timestamp_but_preserves_timestamp_shaped_user_text() {
  let stamp = "2026-10-30T01:00:00.000Z";
  let values = postgres_values(
    "INSERT INTO examples (content,created_at,active) VALUES (?,?,?)",
    vec![stamp.into(), stamp.into(), true.into()],
  );
  assert!(matches!(&values[0], Value::String(_)));
  assert!(matches!(&values[1], Value::ChronoDateTime(_)));
  assert_eq!(values[2], Value::BigInt(Some(1)));
  assert!(matches!(
    postgres_values(
      "UPDATE examples SET finished_at=? WHERE id=? AND locked_until>?",
      vec![stamp.into(), 1i64.into(), stamp.into()]
    )[2],
    Value::ChronoDateTime(_)
  ));
  let counter = postgres_values(
    "INSERT INTO counters(project_id,current_value,created_at,updated_at) VALUES (?,1,?,?) ON CONFLICT(project_id) DO NOTHING",
    vec![1i64.into(), stamp.into(), stamp.into()],
  );
  assert!(matches!(counter[1], Value::ChronoDateTime(_)));
  assert!(matches!(counter[2], Value::ChronoDateTime(_)));
}
