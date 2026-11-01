use super::super::messages::Summary;
use super::*;

#[test]
fn states_follow_server_retry_contract() {
  assert_eq!(State::parse("canceled"), State::Cancelled);
  assert!(State::Cancelled.terminal());
  assert!(State::Succeeded.terminal());
  assert!(!State::Succeeded.retry_job());
  assert!(State::Failed.retry_job());
  assert!(!State::Pending.terminal());
  assert!(!State::Blocked.retry_job());
}
#[test]
fn list_order_uses_iid_and_omits_config_payload() {
  let value = json!([{"id":"9007199254740993","iid":1,"name":"older","config_content":"large"},{"id":"2","iid":5,"status":"failed"}]);
  let rows = pipelines(&value);
  assert_eq!(rows[0].iid, 5);
  assert_eq!(rows[1].id, "9007199254740993");
  assert_eq!(rows[0].state, State::Failed);
  assert!(super::super::messages::activity(&rows).contains("Latest pipeline failed"));
}
#[test]
fn creation_preserves_yaml_and_validates_required_input() {
  let yaml = "jobs:\n  test:\n    script: [cargo test]\n";
  let body = create("web", " main ", " abc ", " .gity-ci.yml ", yaml).unwrap();
  assert_eq!(body["ref_name"], "main");
  assert_eq!(body["config_content"], yaml);
  assert_eq!(body["commit_sha"], "abc");
  assert!(create("push", "main", "", "", yaml).is_err());
  assert!(create("web", " ", "", "", yaml).is_err());
  assert!(create("web", "main", "", "", " ").is_err());
}
#[test]
fn diagnostics_prefer_failed_jobs_over_blocked_and_queued() {
  let make = |status: &str| {
    Job::from(
      &json!({"status":status,"project_job":{"id":"1","status":"pending"},"pipeline_job":{"name":"test"},"artifacts":["reports/*"]}),
    )
  };
  let jobs = vec![make("pending"), make("blocked"), make("failed")];
  let summary = Summary::new(&jobs);
  assert_eq!(summary.total, 3);
  assert_eq!(summary.artifacts, 3);
  assert_eq!(summary.blocked, 1);
  let pipeline = Pipeline::from(&json!({"status":"running"}));
  assert!(summary.guidance(&pipeline, true).contains("failed job log"));
  assert!(
    summary
      .guidance(&pipeline, false)
      .contains("ask a CI maintainer")
  );
  assert!(super::super::messages::next_job(&jobs[1]).contains("runner capacity"));
}
#[test]
fn trace_uses_current_nested_log_and_guards_job_identity() {
  let value = json!({"job":{"id":"123"},"output":"trace","log":{"output":"fallback","duration_millis":77,"exit_code":4}});
  assert_eq!(trace(&value, "123"), Some(("trace".into(), 77, 4)));
  assert_eq!(trace(&value, "124"), None);
  assert_eq!(
    trace(
      &json!({"job":{"id":"123"},"log":{"output":"nested"}}),
      "123"
    )
    .unwrap()
    .0,
    "nested"
  );
}
#[test]
fn timestamps_and_artifact_sizes_handle_unstarted_jobs() {
  assert!(timestamp("0001-01-01T00:00:00Z").is_empty());
  assert_eq!(timestamp("2026-10-30T12:00:00Z"), "2026-10-30T12:00:00Z");
  assert_eq!(bytes(0), "0 B");
  assert_eq!(bytes(1024), "1.0 KB");
  assert_eq!(bytes(1536), "1.5 KB");
}
