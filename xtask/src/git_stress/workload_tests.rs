use super::operations::choose;
use super::process::sanitize;
use crate::git_stress::{Config, Mode, Transport};

pub(in crate::git_stress) fn config(mode: Mode) -> Config {
  Config {
    base_url: String::new(),
    workers: 1,
    iterations: 4,
    duration_seconds: None,
    summary: None,
    transport: Transport::Http,
    ssh_key: None,
    ssh_known_hosts: None,
    ssh_port: 2222,
    ssh_host: None,
    clone_depth: 1,
    bytes_per_file: None,
    seed_commits: 1,
    mode,
    weights: [20, 45, 25, 10],
    timeout_ms: 10,
    workdir: String::new(),
    keep_workdir: false,
    seed_files: 0,
    project_url: String::new(),
    token_env: String::new(),
    token_username: String::new(),
    compose: false,
    build: false,
    project: String::new(),
  }
}

#[test]
fn mixed_workload_selects_each_weighted_operation() {
  let config = config(Mode::Mixed);
  let mut counts = std::collections::BTreeMap::new();
  for iteration in 0..100 {
    *counts.entry(choose(&config, iteration)).or_insert(0) += 1;
  }
  assert_eq!(counts["clone"], 20);
  assert_eq!(counts["fetch"], 45);
  assert_eq!(counts["push"], 25);
  assert_eq!(counts["ls-remote"], 10);
}

#[test]
fn failure_text_redacts_repository_token() {
  assert_eq!(
    sanitize("authentication failed: secret-value", "secret-value"),
    "authentication failed: <token>"
  );
}

#[test]
fn closed_loop_window_ignores_iteration_limit_and_reports_actual_elapsed() {
  let directory = tempfile::tempdir().unwrap();
  let origin = directory.path().join("origin.git");
  assert!(
    std::process::Command::new("git")
      .args(["init", "--bare", "--initial-branch=main"])
      .arg(&origin)
      .output()
      .unwrap()
      .status
      .success()
  );
  let work = directory.path().join("work");
  std::fs::create_dir(&work).unwrap();
  let mut config = config(Mode::LsRemote);
  config.workers = 2;
  config.iterations = 1;
  config.duration_seconds = Some(1);
  config.timeout_ms = 10000;
  let repository = crate::git_stress::api::Repository {
    url: url::Url::from_file_path(&origin).unwrap().to_string(),
    token: String::new(),
    cleanup: None,
  };
  let now = std::time::Instant::now();
  let (summary, failed) = super::execute(
    &config,
    &work,
    "duration-test",
    &repository,
    now,
    std::time::Duration::ZERO,
  )
  .unwrap();
  let value = serde_json::to_value(summary).unwrap();
  assert_eq!(failed, 0);
  assert_eq!(value["measurement"]["requested_duration_seconds"], 1);
  assert!(value["measurement"]["elapsed_seconds"].as_f64().unwrap() >= 1.0);
  assert!(value["measurement"]["count"].as_u64().unwrap() > 1);
  assert_eq!(value["setup"]["operations"]["setup.clone"]["count"], 2);
}
