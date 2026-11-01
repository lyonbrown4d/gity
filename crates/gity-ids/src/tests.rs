use super::{new_id, new_operation_id};

#[test]
fn snowflake_ids_are_positive_ordered_and_unique() {
  let values: Vec<_> = (0..10_000).map(|_| new_id()).collect();
  assert!(values.iter().all(|value| *value > 0));
  assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn operation_ids_are_uuid_v7_and_unique() {
  let values: Vec<_> = (0..1_000).map(|_| new_operation_id()).collect();
  assert!(values.iter().all(|value| value.get_version_num() == 7));
  let unique: std::collections::HashSet<_> = values.iter().collect();
  assert_eq!(unique.len(), values.len());
}
