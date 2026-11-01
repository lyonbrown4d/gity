use super::*;
#[test]
fn unknown_and_alternate_cancelled_statuses_keep_original_behavior() {
  assert_eq!(Status::parse("unknown"), Status::Pending);
  assert_eq!(Status::parse("canceled"), Status::Cancelled);
  assert!(Status::Cancelled.terminal());
  assert!(Status::Succeeded.terminal());
  assert!(!Status::Running.terminal());
}
#[test]
fn queue_diagnostics_prioritize_failures_then_unclaimed_work() {
  let stats = Stats::from_rows(
    &json!([{"status":"failed"},{"status":"pending"},{"status":"running"},{"status":"cancelled"}]),
  );
  assert_eq!(
    stats,
    Stats {
      failed: 1,
      pending: 1,
      running: 1,
      succeeded: 0
    }
  );
  assert!(stats.diagnostic().starts_with("Start with failed jobs"));
  assert!(
    Stats {
      pending: 1,
      ..Default::default()
    }
    .diagnostic()
    .starts_with("Queued jobs with no active workers")
  );
  assert!(
    Stats {
      pending: 1,
      running: 1,
      ..Default::default()
    }
    .diagnostic()
    .starts_with("Running jobs")
  );
}
#[test]
fn enqueue_requires_positive_integer_attempts_and_preserves_payload_text() {
  assert!(enqueue("noop", "", "0").is_err());
  assert!(enqueue("noop", "", "-1").is_err());
  assert!(enqueue("noop", "", "1.5").is_err());
  assert_eq!(
    enqueue("  ", " {\"reason\":\"manual\"} ", "3").unwrap(),
    json!({"kind":"noop","payload":"{\"reason\":\"manual\"}","max_attempts":3})
  );
}
