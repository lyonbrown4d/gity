use axum::{
  Json, Router,
  body::Body,
  extract::{Path, State},
  response::{IntoResponse, Response},
  routing::post,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
  io::{Cursor, Write},
  sync::{Arc, Mutex},
  time::Duration,
};

#[derive(Clone)]
struct Server {
  payload: Value,
  lease_attempt: Option<i64>,
  events: Arc<Mutex<Vec<(String, Value)>>>,
}

async fn endpoint(
  State(state): State<Server>,
  Path(path): Path<String>,
  Json(body): Json<Value>,
) -> Response {
  assert_eq!(body["token"], "runner-test-token");
  state.events.lock().unwrap().push((path.clone(), body));
  let data = match path.as_str() {
    "jobs/claim" => {
      json!({"claimed":true,"job":{"id":"42","attempts":2,"payload":state.payload.to_string()}})
    }
    "heartbeat" => json!({
        "running_job_ids":if state.lease_attempt.is_some() {vec!["42"]} else {vec![]},
        "running_job_attempts":state.lease_attempt.map_or_else(||json!({}), |attempt|json!({"42":attempt})),
    }),
    "jobs/42/source" => {
      let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
      archive
        .start_file("source.txt", zip::write::SimpleFileOptions::default())
        .unwrap();
      archive.write_all(b"repository source\n").unwrap();
      return Body::from(archive.finish().unwrap().into_inner()).into_response();
    }
    _ => json!({}),
  };
  Json(json!({"code":0,"data":data})).into_response()
}

async fn run(
  script: Vec<String>,
  timeout: u64,
  lease_attempt: Option<i64>,
  executor: &str,
) -> (std::process::Output, Vec<(String, Value)>) {
  let root = tempfile::tempdir().unwrap();
  let state = Server {
    payload: json!({"script":script,"commit_sha":"pinned-commit","env":{"CI_VALUE":"value with $literal"},"artifacts":["result.txt"],"timeout_seconds":timeout}),
    lease_attempt,
    events: Arc::default(),
  };
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let url = format!("http://{}", listener.local_addr().unwrap());
  let app = Router::new()
    .route("/api/v1/runners/{*path}", post(endpoint))
    .with_state(state.clone());
  let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
  let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_gity-runner"));
  command
    .args([
      "--server",
      &url,
      "--token",
      "runner-test-token",
      "--executor",
      executor,
      "--once",
      "--work-root",
    ])
    .arg(root.path())
    .kill_on_drop(true);
  let deadline = if executor == "container" { 90 } else { 20 };
  let output = tokio::time::timeout(Duration::from_secs(deadline), command.output())
    .await
    .unwrap()
    .unwrap();
  server.abort();
  assert_eq!(
    std::fs::read_dir(root.path()).unwrap().count(),
    0,
    "job temporary directory must be removed"
  );
  let events = state.events.lock().unwrap().clone();
  for (path, body) in &events {
    if path != "jobs/claim" && path != "heartbeat" {
      assert_eq!(body["attempt"], 2, "{path} must retain claim attempt");
    }
  }
  (output, events)
}

fn trace(events: &[(String, Value)]) -> String {
  events
    .iter()
    .filter(|(path, _)| path.ends_with("/trace"))
    .map(|(_, body)| body["output"].as_str().unwrap_or_default())
    .collect()
}

#[tokio::test]
async fn shell_binary_downloads_source_and_reports_trace_artifact_and_success() {
  let (output, events) = run(
    vec![
      "cat source.txt".into(),
      "printf '%s' \"$CI_VALUE\" > result.txt".into(),
    ],
    10,
    Some(2),
    "shell",
  )
  .await;
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(trace(&events).contains("repository source"));
  let artifact = &events
    .iter()
    .find(|(path, _)| path.ends_with("/artifacts"))
    .unwrap()
    .1;
  assert_eq!(
    STANDARD
      .decode(artifact["content_base64"].as_str().unwrap())
      .unwrap(),
    b"value with $literal"
  );
  let completion = &events
    .iter()
    .find(|(path, _)| path.ends_with("/complete"))
    .unwrap()
    .1;
  assert_eq!(completion["exit_code"], 0);
  assert!(!events.iter().any(|(path, _)| path.ends_with("/fail")));
}

#[tokio::test]
async fn script_failure_reports_exit_code_without_artifact() {
  let (output, events) = run(
    vec!["echo failed-on-purpose >&2; exit 7".into()],
    10,
    Some(2),
    "shell",
  )
  .await;
  assert!(
    output.status.success(),
    "runner successfully reports an unsuccessful script"
  );
  assert!(trace(&events).contains("failed-on-purpose"));
  let failure = &events
    .iter()
    .find(|(path, _)| path.ends_with("/fail"))
    .unwrap()
    .1;
  assert_eq!(failure["exit_code"], 7);
  assert!(
    !events
      .iter()
      .any(|(path, _)| path.ends_with("/artifacts") || path.ends_with("/complete"))
  );
}

#[tokio::test]
async fn deadline_and_lost_lease_terminate_processes_and_report_failure() {
  for (timeout, lease, error) in [
    (1, Some(2), "exceeded its timeout"),
    (10, None, "lease was lost"),
    (10, Some(3), "lease was lost"),
  ] {
    let (output, events) = run(
      vec!["echo started; sleep 30; echo should-not-finish".into()],
      timeout,
      lease,
      "shell",
    )
    .await;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(error));
    assert!(!trace(&events).contains("should-not-finish"));
    assert!(events.iter().any(|(path, _)| path.ends_with("/fail")));
    assert!(!events.iter().any(|(path, _)| path.ends_with("/complete")));
  }
}

#[tokio::test]
#[ignore = "requires a local Docker daemon and the alpine:3.22 image"]
async fn docker_binary_executes_downloaded_source_and_uploads_artifact() {
  let (output, events) = run(
    vec![
      "cat source.txt".into(),
      "printf '%s' \"$CI_VALUE\" > result.txt".into(),
    ],
    60,
    Some(2),
    "container",
  )
  .await;
  assert!(
    output.status.success(),
    "{} trace={}",
    String::from_utf8_lossy(&output.stderr),
    trace(&events)
  );
  assert!(trace(&events).contains("repository source"));
  let artifact = &events
    .iter()
    .find(|(path, _)| path.ends_with("/artifacts"))
    .unwrap()
    .1;
  assert_eq!(
    STANDARD
      .decode(artifact["content_base64"].as_str().unwrap())
      .unwrap(),
    b"value with $literal"
  );
  assert!(events.iter().any(|(path, _)| path.ends_with("/complete")));
}
