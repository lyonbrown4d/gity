use super::*;
use crate::git_stress::{Mode, workload::tests::config};
use std::sync::{Arc, Mutex};
#[test]
fn report_separates_setup_and_counts_failed_measurements_with_nearest_rank_tails() {
  let stats = Arc::new(Mutex::new(BTreeMap::from([
    (
      "setup.clone".into(),
      vec![(Duration::from_millis(999), true)],
    ),
    (
      "push".into(),
      vec![
        (Duration::from_millis(10), true),
        (Duration::from_millis(20), false),
        (Duration::from_millis(100), true),
      ],
    ),
  ])));
  let report = Summary::new(
    &config(Mode::Push),
    "run",
    &stats,
    Duration::from_secs(2),
    Duration::from_millis(1500),
    (10.0, 11.5),
  );
  assert_eq!(report.measurement.count, 3);
  assert_eq!(report.measurement.fail, 1);
  assert_eq!(report.measurement.throughput_ops_per_second, 2.0);
  let push = &report.measurement.operations["push"];
  assert_eq!(push.p50_ms, 20.0);
  assert_eq!(push.p95_ms, 100.0);
  assert_eq!(push.p99_ms, 100.0);
  assert_eq!(push.max_ms, 100.0);
  assert_eq!(report.setup.count, 1);
  assert_eq!(report.setup.elapsed_seconds, 2.0);
  let json = serde_json::to_value(report).unwrap();
  assert!(json.get("token").is_none());
  assert_eq!(json["measurement"]["operations"]["push"]["fail"], 1);
  assert_eq!(json["measurement"]["started_unix_seconds"], 10.0);
  assert_eq!(json["measurement"]["finished_unix_seconds"], 11.5);
  assert!(json["setup"].get("started_unix_seconds").is_none());
}
