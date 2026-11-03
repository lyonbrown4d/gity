use axum::{
  body::{Body, to_bytes},
  http::Request,
};
use gity_core::Settings;
use gity_platform::{AppState, compose, router};
use russh::keys::{PrivateKey, ssh_key::LineEnding};
use serde_json::{Value, json};
use std::{
  path::{Path, PathBuf},
  process::Output,
};
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;

pub struct Fixture {
  pub root: tempfile::TempDir,
  pub state: AppState,
  pub url: String,
  pub port: u16,
  pub known_hosts: PathBuf,
  pub shutdown: CancellationToken,
  pub server: tokio::task::JoinHandle<anyhow::Result<()>>,
}
impl Fixture {
  pub async fn new() -> Self {
    let root = tempfile::tempdir().unwrap();
    let mut settings = Settings::try_load(None).unwrap();
    settings.database_url = "sqlite::memory:".into();
    settings.repo_root = root.path().join("repos");
    settings.storage.root = root.path().join("storage");
    settings.storage.driver = "local".into();
    settings.search.index_root = root.path().join("search");
    settings.search.code_index_root = root.path().join("code-index");
    settings.search.index_enabled = false;
    settings.ssh.enabled = true;
    settings.ssh.host_key = root.path().join("host_ed25519");
    settings.git_hook = Some(env!("CARGO_BIN_EXE_gity-server").into());
    let (_, state) = compose(settings).await.unwrap();
    state.db.migrate().await.unwrap();
    insert(
      &state,
      "users",
      json!({"id":1,"username":"owner","display_name":"Owner","is_super_admin":1}),
    )
    .await;
    insert(
      &state,
      "organizations",
      json!({"id":10,"name":"Team","path_key":"team","full_path":"team","visibility":"private"}),
    )
    .await;
    for (id, name) in [(20, "demo"), (30, "other")] {
      insert(&state,"projects",json!({"id":id,"organization_id":10,"name":name,"path_key":name,"full_path":format!("team/{name}"),"visibility":"private","default_branch":"main","status":"active"})).await;
      state
        .git
        .init_bare(&format!("{id}.git"), "main")
        .await
        .unwrap();
    }
    state
      .git
      .create_file_commit(
        "20.git",
        gity_git::CreateFileCommitInput {
          expected_head_sha: None,
          update_existing: false,
          branch_name: "main".into(),
          file_path: "README.md".into(),
          content: "base\n".into(),
          message: "initial".into(),
          author_name: "Owner".into(),
          author_email: "owner@example.test".into(),
        },
      )
      .await
      .unwrap();
    state.db.execute("INSERT INTO auth_sessions(token_hash,user_id,kind,expires_at,family) VALUES (?,1,'access',?,'ssh-test')",vec![gity_platform::auth::token_hash("ssh-api-token").into(),"2037-01-01T00:00:00Z".into()]).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let config = gity_platform::ssh::configuration(&state).unwrap();
    let known_hosts = root.path().join("known_hosts");
    std::fs::write(
      &known_hosts,
      format!(
        "[127.0.0.1]:{port} {}\n",
        config.keys[0].public_key().to_openssh().unwrap()
      ),
    )
    .unwrap();
    let shutdown = CancellationToken::new();
    let server = tokio::spawn(gity_platform::ssh::run(
      state.clone(),
      listener,
      shutdown.clone(),
      config,
    ));
    Self {
      root,
      state,
      url: format!("ssh://git@127.0.0.1:{port}/team/demo.git"),
      port,
      known_hosts,
      shutdown,
      server,
    }
  }
  pub async fn api(&self, method: &str, path: &str, body: Value) -> (u16, Value) {
    let request = Request::builder()
      .method(method)
      .uri(format!("/api/v1{path}"))
      .header("authorization", "Bearer ssh-api-token")
      .header("content-type", "application/json")
      .body(Body::from(body.to_string()))
      .unwrap();
    let response = router(self.state.clone()).oneshot(request).await.unwrap();
    let status = response.status().as_u16();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
  }
  pub async fn key(&self, name: &str, can_push: bool) -> (PathBuf, String) {
    let path = self.root.path().join(name);
    let key = PrivateKey::random(&mut rand::rng(), russh::keys::Algorithm::Ed25519).unwrap();
    std::fs::write(&path, key.to_openssh(LineEnding::LF).unwrap().as_bytes()).unwrap();
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let public = key.public_key().to_openssh().unwrap();
    let response = self
      .api(
        "POST",
        "/projects/20/deploy-keys",
        json!({"title":name,"public_key":public,"can_push":can_push}),
      )
      .await;
    assert_eq!(response.0, 200, "{}", response.1);
    (path, response.1["data"]["id"].as_str().unwrap().into())
  }
  pub async fn git(&self, dir: &Path, key: &Path, args: &[&str]) -> Output {
    let ssh = format!(
      "ssh -F /dev/null -i '{}' -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile='{}' -o ConnectTimeout=5",
      key.display(),
      self.known_hosts.display()
    );
    tokio::time::timeout(
      std::time::Duration::from_secs(20),
      tokio::process::Command::new("git")
        .current_dir(dir)
        .env("GIT_SSH_COMMAND", ssh)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(args)
        .kill_on_drop(true)
        .output(),
    )
    .await
    .unwrap()
    .unwrap()
  }
  pub async fn ok(&self, dir: &Path, key: &Path, args: &[&str]) -> String {
    let output = self.git(dir, key, args).await;
    assert!(
      output.status.success(),
      "git {args:?}: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().into()
  }
  pub async fn ssh(&self, key: &Path, args: &[&str]) -> Output {
    let mut command = tokio::process::Command::new("ssh");
    command
      .args([
        "-F",
        "/dev/null",
        "-o",
        "IdentitiesOnly=yes",
        "-o",
        "BatchMode=yes",
        "-o",
        "StrictHostKeyChecking=yes",
        "-o",
        "ConnectTimeout=5",
      ])
      .arg("-i")
      .arg(key)
      .arg("-p")
      .arg(self.port.to_string())
      .arg("-o")
      .arg(format!("UserKnownHostsFile={}", self.known_hosts.display()));
    if args.first() == Some(&"-s") {
      command.arg("-s").arg("git@127.0.0.1").args(&args[1..]);
    } else {
      command.arg("git@127.0.0.1").args(args);
    }
    tokio::time::timeout(
      std::time::Duration::from_secs(10),
      command
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true)
        .output(),
    )
    .await
    .unwrap()
    .unwrap()
  }
  pub async fn stop(self) {
    self.shutdown.cancel();
    tokio::time::timeout(std::time::Duration::from_secs(5), self.server)
      .await
      .unwrap()
      .unwrap()
      .unwrap();
  }
}
async fn insert(state: &AppState, table: &str, body: Value) {
  state
    .db
    .insert(table, body.as_object().unwrap().clone())
    .await
    .unwrap();
}
