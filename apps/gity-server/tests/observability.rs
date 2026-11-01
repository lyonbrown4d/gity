use axum::{Router, body::Bytes, routing::post};
use std::{process::Stdio, time::Duration};
use tokio::{process::Command, sync::mpsc};

/// Exercise the packaged process rather than a mocked tracing layer: the SDK
/// batch thread must be able to export HTTP outside a Tokio reactor.
#[tokio::test]
async fn packaged_server_exports_request_trace_with_remote_context() {
  let directory = tempfile::tempdir().unwrap();
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let collector = listener.local_addr().unwrap();
  let (sender, mut received) = mpsc::channel::<Bytes>(8);
  let app = Router::new().route(
    "/v1/traces",
    post(move |body: Bytes| {
      let sender = sender.clone();
      async move {
        let _ = sender.send(body).await;
        [("content-type", "application/x-protobuf")]
      }
    }),
  );
  let collector_task = tokio::spawn(async move {
    axum::serve(listener, app).await.unwrap();
  });
  let available = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
  let address = available.local_addr().unwrap();
  drop(available);
  let log = std::fs::File::create(directory.path().join("server.log")).unwrap();
  let mut child = Command::new(env!("CARGO_BIN_EXE_gity-server"))
    .arg("standalone")
    .env("RUST_LOG", "gity=info,sqlx=warn,opentelemetry_sdk=warn")
    .env("GITY_HTTP__ADDRESS", address.to_string())
    .env("GITY_DATABASE__DRIVER", "sqlite")
    .env(
      "GITY_DATABASE__DSN",
      format!(
        "sqlite://{}?mode=rwc",
        directory.path().join("db.sqlite").display()
      ),
    )
    .env("GITY_GIT__REPO_ROOT", directory.path().join("repos"))
    .env("GITY_STORAGE__DRIVER", "local")
    .env("GITY_STORAGE__ROOT", directory.path().join("objects"))
    .env("GITY_SEARCH__INDEX_ROOT", directory.path().join("indexes"))
    .env("GITY_SEARCH__INDEX_ENABLED", "false")
    .env("GITY_OBSERVABILITY__METRICS_ENABLED", "false")
    .env(
      "GITY_OBSERVABILITY__OTLP_ENDPOINT",
      format!("http://{collector}/v1/traces"),
    )
    .env("OTEL_BSP_SCHEDULE_DELAY", "100")
    .stdin(Stdio::null())
    .stdout(log.try_clone().unwrap())
    .stderr(log)
    .kill_on_drop(true)
    .spawn()
    .unwrap();
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(2))
    .build()
    .unwrap();
  let health = format!("http://{address}/api/health");
  tokio::time::timeout(Duration::from_secs(15), async {
    loop {
      if let Some(status) = child.try_wait().unwrap() {
        panic!(
          "server exited {status}: {}",
          std::fs::read_to_string(directory.path().join("server.log")).unwrap()
        );
      }
      if client
        .get(&health)
        .send()
        .await
        .is_ok_and(|v| v.status().is_success())
      {
        break;
      }
      tokio::time::sleep(Duration::from_millis(50)).await;
    }
  })
  .await
  .expect("server startup timed out");
  let trace_id = "abcdef1234567890abcdef1234567890";
  let response = client
    .get(&health)
    .header("traceparent", format!("00-{trace_id}-abcdef1234567890-01"))
    .header("x-request-id", "integration-trace")
    .send()
    .await
    .unwrap();
  assert_eq!(response.status(), 200);
  assert_eq!(response.headers()["x-request-id"], "integration-trace");
  let wanted = hex::decode(trace_id).unwrap();
  let exported = tokio::time::timeout(Duration::from_secs(10), async {
    while let Some(body) = received.recv().await {
      if body.windows(wanted.len()).any(|v| v == wanted) {
        return body;
      }
    }
    panic!("collector closed before the request trace arrived");
  })
  .await;
  let logs = std::fs::read_to_string(directory.path().join("server.log")).unwrap();
  child.kill().await.unwrap();
  child.wait().await.unwrap();
  collector_task.abort();
  let body = exported.unwrap_or_else(|_| panic!("trace was not exported: {logs}"));
  assert!(
    body
      .windows("http.request".len())
      .any(|v| v == b"http.request")
  );
  assert!(!logs.contains("no reactor running"));
  assert!(!logs.contains("AfterShutdown"));
}
