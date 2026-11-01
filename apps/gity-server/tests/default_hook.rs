#![cfg(target_os = "linux")]

use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::{
  fs,
  net::TcpListener,
  process::{Child, Command, Stdio},
  thread,
  time::{Duration, Instant},
};

struct Server(Child);
impl Drop for Server {
  fn drop(&mut self) {
    let _ = Command::new("kill")
      .args(["-TERM", &self.0.id().to_string()])
      .status();
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
      if self.0.try_wait().ok().flatten().is_some() {
        return;
      }
      thread::sleep(Duration::from_millis(20));
    }
    let _ = self.0.kill();
    let _ = self.0.wait();
  }
}

#[test]
fn default_hook_remains_usable_after_running_server_binary_is_replaced() {
  let root = tempfile::tempdir().unwrap();
  let binary = root.path().join("gity-server");
  fs::copy(env!("CARGO_BIN_EXE_gity-server"), &binary).unwrap();
  let socket = TcpListener::bind("127.0.0.1:0").unwrap();
  let address = socket.local_addr().unwrap();
  drop(socket);
  let base = format!("http://{address}");
  let mut server = Server(
    Command::new(&binary)
      .arg("standalone")
      .env("GITY_HTTP__ADDRESS", address.to_string())
      .env("GITY_HTTP__BASE_URL", &base)
      .env("GITY_DATABASE__DRIVER", "sqlite")
      .env(
        "GITY_DATABASE__DSN",
        format!(
          "sqlite://{}?mode=rwc",
          root.path().join("test.db").display()
        ),
      )
      .env("GITY_GIT__REPO_ROOT", root.path().join("repos"))
      .env_remove("GITY_GIT__HOOK")
      .env("GITY_STORAGE__DRIVER", "local")
      .env("GITY_STORAGE__ROOT", root.path().join("storage"))
      .env("GITY_SEARCH__INDEX_ROOT", root.path().join("index"))
      .env("GITY_SEARCH__INDEX_ENABLED", "false")
      .env("GITY_OBSERVABILITY__METRICS_ENABLED", "false")
      .env("GITY_OBSERVABILITY__OTLP_ENDPOINT", "")
      .env("GITY_SSH__ENABLED", "false")
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
      .unwrap(),
  );
  let client = Client::builder()
    .timeout(Duration::from_secs(5))
    .no_proxy()
    .build()
    .unwrap();
  let deadline = Instant::now() + Duration::from_secs(10);
  loop {
    if client
      .get(format!("{base}/api/health"))
      .send()
      .is_ok_and(|response| response.status().is_success())
    {
      break;
    }
    assert!(server.0.try_wait().unwrap().is_none(), "server exited");
    assert!(Instant::now() < deadline, "server did not become healthy");
    thread::sleep(Duration::from_millis(20));
  }
  let post = |path: &str, token: &str, body: Value| -> Value {
    let response = client
      .post(format!("{base}/api/v1{path}"))
      .bearer_auth(token)
      .json(&body)
      .send()
      .unwrap();
    assert!(response.status().is_success());
    response.json::<Value>().unwrap()["data"].clone()
  };
  let login = post(
    "/auth/login",
    "",
    json!({"username":"default-hook","password":"default-hook-local-only"}),
  );
  let token = login["token"].as_str().unwrap();
  let orgs: Value = client
    .get(format!("{base}/api/v1/orgs"))
    .bearer_auth(token)
    .send()
    .unwrap()
    .json()
    .unwrap();
  let project = post(
    "/projects",
    token,
    json!({"organization_id":orgs["data"][0]["id"],"name":"hook","key":"hook"}),
  );
  let path = format!(
    "/projects/{}/repository/files",
    project["id"].as_str().unwrap()
  );
  post(
    &path,
    token,
    json!({"branch_name":"main","file_path":"README.md","content":"hook regression\n","message":"initial"}),
  );
  let replacement = root.path().join("next-server");
  fs::copy(env!("CARGO_BIN_EXE_gity-server"), &replacement).unwrap();
  fs::rename(replacement, &binary).unwrap();
  let response = client
    .get(format!(
      "{base}/{}.git/info/refs?service=git-receive-pack",
      project["full_path"].as_str().unwrap()
    ))
    .basic_auth("git", Some(token))
    .send()
    .unwrap();
  assert_eq!(response.status(), 200);
  assert_eq!(
    response.headers()["content-type"],
    "application/x-git-receive-pack-advertisement"
  );
  let bytes = response.bytes().unwrap();
  assert!(
    bytes
      .windows(b"refs/heads/main".len())
      .any(|reference| reference == b"refs/heads/main")
  );
}
