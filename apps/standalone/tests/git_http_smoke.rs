use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::fs::{self, File};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tempfile::TempDir;
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
struct AuthResponse {
  token: String,
  organization_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RepositoryResponse {
  id: String,
}

#[derive(Debug, Deserialize)]
struct BranchView {
  name: String,
  last_commit_sha: Option<String>,
}

struct RunningServer {
  child: Child,
  stdout_log: PathBuf,
  stderr_log: PathBuf,
}

impl RunningServer {
  fn spawn(
    standalone_bin: &Path,
    project_root: &Path,
    port: u16,
    repo_root: &Path,
    stdout_log: PathBuf,
    stderr_log: PathBuf,
  ) -> Result<Self, String> {
    let stdout = File::create(&stdout_log)
      .map_err(|err| format!("failed to create server stdout log file: {err}"))?;
    let stderr = File::create(&stderr_log)
      .map_err(|err| format!("failed to create server stderr log file: {err}"))?;

    let mut cmd = Command::new(standalone_bin);
    cmd
      .current_dir(project_root)
      .env("GITY_SERVER_PORT", port.to_string())
      .env("GITY_STORAGE_REPO_ROOT", repo_root)
      .env(
        "GITY_DATABASE_URL",
        std::env::var("GITY_DATABASE_URL")
          .unwrap_or_else(|_| "postgres://root:root@localhost:5433/gity".to_string()),
      )
      .env(
        "GITY_CACHE_CACHE_TYPE",
        std::env::var("GITY_CACHE_CACHE_TYPE").unwrap_or_else(|_| "REDIS".to_string()),
      )
      .env(
        "GITY_CACHE_URL",
        std::env::var("GITY_CACHE_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
      )
      .stdout(Stdio::from(stdout))
      .stderr(Stdio::from(stderr));

    let child = cmd
      .spawn()
      .map_err(|err| format!("failed to start standalone process: {err}"))?;
    Ok(Self {
      child,
      stdout_log,
      stderr_log,
    })
  }

  fn stop(&mut self) {
    if self.child.try_wait().ok().flatten().is_none() {
      let _ = self.child.kill();
      let _ = self.child.wait();
    }
  }

  fn logs_tail(&self) -> String {
    fn tail(path: &Path) -> String {
      match fs::read_to_string(path) {
        Ok(content) => {
          let lines: Vec<&str> = content.lines().collect();
          let start = lines.len().saturating_sub(50);
          lines[start..].join("\n")
        }
        Err(err) => format!("failed to read log {}: {err}", path.display()),
      }
    }

    format!(
      "----- server stdout tail -----\n{}\n----- server stderr tail -----\n{}",
      tail(&self.stdout_log),
      tail(&self.stderr_log)
    )
  }
}

impl Drop for RunningServer {
  fn drop(&mut self) {
    self.stop();
  }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires docker-compose postgres/redis and local git binary"]
async fn git_http_smoke_push_and_branch_sync() {
  if let Err(err) = run_smoke().await {
    panic!("{err}");
  }
}

async fn run_smoke() -> Result<(), String> {
  let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
  let standalone_bin = resolve_standalone_bin()?;
  let port = find_free_port()?;
  let run_id = unique_id();
  let organization_key = format!("org{run_id}");
  let repository_key = format!("demo{run_id}");
  let username = format!("u{run_id}");
  let email = format!("u{run_id}@example.com");
  let repo_storage =
    TempDir::new().map_err(|err| format!("failed to create temp repo root: {err}"))?;
  let git_workdir =
    TempDir::new().map_err(|err| format!("failed to create temp git dir: {err}"))?;
  let logs_dir = TempDir::new().map_err(|err| format!("failed to create temp log dir: {err}"))?;
  let stdout_log = logs_dir.path().join("standalone.stdout.log");
  let stderr_log = logs_dir.path().join("standalone.stderr.log");

  let mut server = RunningServer::spawn(
    &standalone_bin,
    &project_root,
    port,
    repo_storage.path(),
    stdout_log,
    stderr_log,
  )?;

  let client = Client::builder()
    .timeout(Duration::from_secs(8))
    .build()
    .map_err(|err| format!("failed to build reqwest client: {err}"))?;

  if let Err(err) = wait_ready(&client, port).await {
    return Err(format!("{err}\n{}", server.logs_tail()));
  }

  let register: AuthResponse = post_json(
    &client,
    &format!("http://127.0.0.1:{port}/api/v1/auth/register"),
    None,
    json!({
      "username": username,
      "email": email,
      "password": "Passw0rd!",
      "organization_name": format!("Org {run_id}"),
      "organization_key": organization_key,
    }),
  )
  .await
  .map_err(|err| format!("{err}\n{}", server.logs_tail()))?;

  let token = register.token;
  let organization_id = register
    .organization_id
    .ok_or_else(|| "register response missing organization_id".to_string())?;

  let repo: RepositoryResponse = post_json(
    &client,
    &format!("http://127.0.0.1:{port}/api/v1/repos"),
    Some(token.as_str()),
    json!({
      "organization_id": organization_id,
      "key": repository_key,
      "name": format!("Demo {run_id}"),
      "default_branch": "main",
    }),
  )
  .await
  .map_err(|err| format!("{err}\n{}", server.logs_tail()))?;

  let remote = format!("http://127.0.0.1:{port}/git/{organization_key}/{repository_key}.git");
  let git_root = git_workdir.path();

  git_ok(git_root, &["init", "-b", "main"])?;
  git_ok(git_root, &["config", "user.name", "Smoke Bot"])?;
  git_ok(git_root, &["config", "user.email", "smoke@example.com"])?;
  fs::write(git_root.join("README.md"), format!("smoke {run_id}\n"))
    .map_err(|err| format!("failed to write README.md: {err}"))?;
  git_ok(git_root, &["add", "README.md"])?;
  git_ok(git_root, &["commit", "-m", "init"])?;
  let commit1 = git_stdout(git_root, &["rev-parse", "HEAD"])?;
  git_ok(git_root, &["remote", "add", "origin", remote.as_str()])?;
  let auth_header = format!("http.extraHeader=Authorization: Bearer {token}");
  let _auth_push1 = git(
    git_root,
    &["-c", auth_header.as_str(), "push", "-u", "origin", "main"],
  )
  .map_err(|err| format!("{err}\n{}", server.logs_tail()))?;

  let ls1 = git_stdout(
    project_root.as_path(),
    &[
      "-c",
      auth_header.as_str(),
      "ls-remote",
      remote.as_str(),
      "refs/heads/main",
    ],
  )?;
  let ls1_sha = parse_ls_remote_sha(ls1.as_str())?;
  let branches1: Vec<BranchView> = get_json(
    &client,
    &format!("http://127.0.0.1:{port}/api/v1/repos/{}/branches", repo.id),
    Some(token.as_str()),
  )
  .await
  .map_err(|err| format!("{err}\n{}", server.logs_tail()))?;
  let meta1 = find_main_branch_sha(&branches1)?;
  if commit1 != ls1_sha || commit1 != meta1 {
    return Err(format!(
      "first push verification failed: commit={commit1} ls_remote={ls1_sha} metadata={meta1}"
    ));
  }

  fs::write(
    git_root.join("README.md"),
    format!("smoke second {run_id}\n"),
  )
  .map_err(|err| format!("failed to update README.md: {err}"))?;
  git_ok(git_root, &["add", "README.md"])?;
  git_ok(git_root, &["commit", "-m", "second"])?;
  let commit2 = git_stdout(git_root, &["rev-parse", "HEAD"])?;

  let unauthorized = git(git_root, &["push", "origin", "main"])
    .map_err(|err| format!("{err}\n{}", server.logs_tail()))?;
  if unauthorized.status.success() {
    return Err("unauthorized push unexpectedly succeeded".to_string());
  }

  let ls_after_unauth = git_stdout(
    project_root.as_path(),
    &[
      "-c",
      auth_header.as_str(),
      "ls-remote",
      remote.as_str(),
      "refs/heads/main",
    ],
  )?;
  let ls_after_unauth_sha = parse_ls_remote_sha(ls_after_unauth.as_str())?;
  if ls_after_unauth_sha != commit1 {
    return Err(format!(
      "unauthorized push changed remote ref unexpectedly: before={commit1} after={ls_after_unauth_sha}"
    ));
  }

  let _auth_push2 = git(
    git_root,
    &["-c", auth_header.as_str(), "push", "origin", "main"],
  )
  .map_err(|err| format!("{err}\n{}", server.logs_tail()))?;

  let ls2 = git_stdout(
    project_root.as_path(),
    &[
      "-c",
      auth_header.as_str(),
      "ls-remote",
      remote.as_str(),
      "refs/heads/main",
    ],
  )?;
  let ls2_sha = parse_ls_remote_sha(ls2.as_str())?;
  let branches2: Vec<BranchView> = get_json(
    &client,
    &format!("http://127.0.0.1:{port}/api/v1/repos/{}/branches", repo.id),
    Some(token.as_str()),
  )
  .await
  .map_err(|err| format!("{err}\n{}", server.logs_tail()))?;
  let meta2 = find_main_branch_sha(&branches2)?;
  if commit2 != ls2_sha || commit2 != meta2 {
    return Err(format!(
      "second push verification failed: commit={commit2} ls_remote={ls2_sha} metadata={meta2}"
    ));
  }

  server.stop();
  Ok(())
}

fn resolve_standalone_bin() -> Result<PathBuf, String> {
  if let Ok(path) = std::env::var("CARGO_BIN_EXE_standalone") {
    let candidate = PathBuf::from(path);
    if candidate.exists() {
      return Ok(candidate);
    }
  }

  let current_exe = std::env::current_exe()
    .map_err(|err| format!("failed to read current test executable path: {err}"))?;
  let debug_dir = current_exe
    .parent()
    .and_then(|path| path.parent())
    .ok_or_else(|| {
      format!(
        "failed to derive target debug dir from test executable path: {}",
        current_exe.display()
      )
    })?;
  let binary_name = if cfg!(windows) {
    "standalone.exe"
  } else {
    "standalone"
  };
  let inferred = debug_dir.join(binary_name);
  if inferred.exists() {
    return Ok(inferred);
  }

  Err(format!(
    "standalone binary not found. expected one of:\n- env CARGO_BIN_EXE_standalone\n- {}\nrun `CARGO_TARGET_DIR=target_alt cargo build -p standalone` first",
    inferred.display()
  ))
}

async fn wait_ready(client: &Client, port: u16) -> Result<(), String> {
  let url = format!("http://127.0.0.1:{port}/api-docs/openapi.json");
  for _ in 0..120 {
    match client.get(url.as_str()).send().await {
      Ok(resp) if resp.status().is_success() => return Ok(()),
      _ => sleep(Duration::from_millis(500)).await,
    }
  }
  Err(format!("standalone did not become ready on port {port}"))
}

async fn post_json<T: for<'de> Deserialize<'de>>(
  client: &Client,
  url: &str,
  token: Option<&str>,
  body: serde_json::Value,
) -> Result<T, String> {
  let mut req = client.post(url).json(&body);
  if let Some(token) = token {
    req = req.bearer_auth(token);
  }
  let resp = req
    .send()
    .await
    .map_err(|err| format!("POST {url} failed: {err}"))?;
  let status = resp.status();
  let text = resp
    .text()
    .await
    .map_err(|err| format!("POST {url} read body failed: {err}"))?;
  if !status.is_success() {
    return Err(format!("POST {url} failed: status={status}, body={text}"));
  }
  serde_json::from_str(text.as_str())
    .map_err(|err| format!("POST {url} parse json failed: {err}, body={text}"))
}

async fn get_json<T: for<'de> Deserialize<'de>>(
  client: &Client,
  url: &str,
  token: Option<&str>,
) -> Result<T, String> {
  let mut req = client.get(url);
  if let Some(token) = token {
    req = req.bearer_auth(token);
  }
  let resp = req
    .send()
    .await
    .map_err(|err| format!("GET {url} failed: {err}"))?;
  let status = resp.status();
  let text = resp
    .text()
    .await
    .map_err(|err| format!("GET {url} read body failed: {err}"))?;
  if !status.is_success() {
    return Err(format!("GET {url} failed: status={status}, body={text}"));
  }
  serde_json::from_str(text.as_str())
    .map_err(|err| format!("GET {url} parse json failed: {err}, body={text}"))
}

fn git(cwd: &Path, args: &[&str]) -> Result<Output, String> {
  Command::new("git")
    .arg("-C")
    .arg(cwd)
    .args(args)
    .output()
    .map_err(|err| {
      format!(
        "failed to run git command: git -C {} {:?}: {err}",
        cwd.display(),
        args
      )
    })
}

fn git_ok(cwd: &Path, args: &[&str]) -> Result<(), String> {
  let output = git(cwd, args)?;
  if output.status.success() {
    return Ok(());
  }
  Err(format!(
    "git command failed: git -C {} {:?}\nstdout:\n{}\nstderr:\n{}",
    cwd.display(),
    args,
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr)
  ))
}

fn git_stdout(cwd: &Path, args: &[&str]) -> Result<String, String> {
  let output = git(cwd, args)?;
  if !output.status.success() {
    return Err(format!(
      "git command failed: git -C {} {:?}\nstdout:\n{}\nstderr:\n{}",
      cwd.display(),
      args,
      String::from_utf8_lossy(&output.stdout),
      String::from_utf8_lossy(&output.stderr)
    ));
  }
  Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn parse_ls_remote_sha(line: &str) -> Result<String, String> {
  let first = line
    .split_whitespace()
    .next()
    .ok_or_else(|| format!("invalid ls-remote output: {line}"))?;
  Ok(first.to_string())
}

fn find_main_branch_sha(branches: &[BranchView]) -> Result<String, String> {
  let main = branches
    .iter()
    .find(|branch| branch.name == "main")
    .ok_or_else(|| "main branch not found in branch metadata".to_string())?;
  main
    .last_commit_sha
    .clone()
    .ok_or_else(|| "main branch last_commit_sha is missing".to_string())
}

fn find_free_port() -> Result<u16, String> {
  let listener = TcpListener::bind("127.0.0.1:0")
    .map_err(|err| format!("failed to bind random local port: {err}"))?;
  listener
    .local_addr()
    .map(|addr| addr.port())
    .map_err(|err| format!("failed to read random local port: {err}"))
}

fn unique_id() -> String {
  let now_ms = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis();
  format!("{now_ms}{}", std::process::id())
}
