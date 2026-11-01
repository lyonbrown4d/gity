use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ferroid::time::TimeSource;

use super::{IdError, IdGenerator, MAX_NODE_ID, install, new_id, new_operation_id};

#[derive(Clone)]
struct FakeClock(Arc<AtomicU64>);

impl FakeClock {
  fn at(millis: u64) -> Self {
    Self(Arc::new(AtomicU64::new(millis)))
  }
  fn set(&self, millis: u64) {
    self.0.store(millis, Ordering::Relaxed);
  }
}

impl TimeSource<u64> for FakeClock {
  fn current_millis(&self) -> u64 {
    self.0.load(Ordering::Relaxed)
  }
}

#[test]
fn snowflake_ids_are_positive_ordered_and_unique() {
  let values: Vec<_> = (0..10_000).map(|_| new_id()).collect();
  assert!(values.iter().all(|value| *value > 0));
  assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn one_millisecond_holds_a_full_sequence_then_exhausts() {
  let generator = IdGenerator::with_clock(3, FakeClock::at(1_000_000)).unwrap();
  let values: Vec<i64> = (0..4_096)
    .map(|_| generator.next().expect("sequence has 4096 slots per node"))
    .collect();
  assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
  assert_eq!(generator.next(), Err(IdError::Exhausted));
}

#[test]
fn clock_rollback_beyond_tolerance_is_reported() {
  let clock = FakeClock::at(1_000_000);
  let generator = IdGenerator::with_clock(4, clock.clone()).unwrap();
  generator.next().unwrap();
  clock.set(997_000);
  assert_eq!(generator.next(), Err(IdError::ClockBehind(3_000)));
}

#[test]
fn node_bits_are_encoded_in_the_identifier() {
  let generator = IdGenerator::new(MAX_NODE_ID).unwrap();
  let raw = u64::try_from(generator.next().unwrap()).unwrap();
  assert_eq!((raw >> 12) & 0x3FF, u64::from(MAX_NODE_ID));
  assert!(raw >> 22 > 0);
}

#[test]
fn node_id_out_of_range_is_rejected() {
  assert!(matches!(
    IdGenerator::new(MAX_NODE_ID + 1),
    Err(IdError::InvalidNodeId(1_024))
  ));
}

#[test]
fn install_rejects_a_conflicting_node() {
  let _ = new_id();
  assert!(install(IdGenerator::new(1).unwrap()).is_ok());
  assert!(matches!(
    install(IdGenerator::new(2).unwrap()),
    Err(IdError::NodeConflict {
      installed: 1,
      requested: 2
    })
  ));
}

#[test]
fn operation_ids_are_uuid_v7_and_unique() {
  let values: Vec<_> = (0..1_000).map(|_| new_operation_id()).collect();
  assert!(values.iter().all(|value| value.get_version_num() == 7));
  let unique: std::collections::HashSet<_> = values.iter().collect();
  assert_eq!(unique.len(), values.len());
}
