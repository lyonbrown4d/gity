use base64::{Engine, engine::general_purpose::STANDARD};
use gity_core::Settings;
use gity_platform::{AppState, compose, router};
use serde_json::{Value, json};
use std::{path::Path, process::Output};

async fn insert(s: &AppState, table: &str, value: Value) -> Value {
  s.db
    .insert(table, value.as_object().unwrap().clone())
    .await
    .unwrap()
}

async fn fixture(root: &Path) -> AppState {
  let mut settings = Settings::try_load(None).unwrap();
  settings.database_url = "sqlite::memory:".into();
  settings.repo_root = root.join("repos");
  settings.storage_root = root.join("storage");
  settings.storage.driver = "local".into();
  settings.storage.root = settings.storage_root.clone();
  settings.search.index_root = root.join("search");
  settings.search.code_index_root = root.join("code-index");
  settings.search.index_enabled = false;
  settings.git_hook = Some(env!("CARGO_BIN_EXE_gity-server").into());
  let (_module, state) = compose(settings).await.unwrap();
  state.db.migrate().await.unwrap();
  let user = insert(&state, "users", json!({"username":"owner","display_name":"Owner","email":"owner@example.test","is_super_admin":1})).await;
  let org = insert(&state,"organizations",json!({"name":"Team","path_key":"team","full_path":"team","description":"","visibility":"private"})).await;
  insert(&state,"projects",json!({"id":20,"organization_id":org["id"],"name":"Demo","path_key":"demo","full_path":"team/demo","description":"","visibility":"private","default_branch":"main","status":"active"})).await;
  state
    .db
    .execute(
      "INSERT INTO auth_sessions(token_hash,user_id,kind,expires_at,family) VALUES (?,?,?,?,?)",
      vec![
        gity_platform::auth::token_hash("git-test-token").into(),
        user["id"].as_i64().unwrap().into(),
        "access".into(),
        "2999-01-01T00:00:00Z".into(),
        "git-http-test".into(),
      ],
    )
    .await
    .unwrap();
  state.git.init_bare("20.git", "main").await.unwrap();
  state
    .git
    .create_file_commit(
      "20.git",
      gity_git::CreateFileCommitInput {
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
  state
}

async fn git(dir: &Path, args: &[&str], authenticated: bool) -> Output {
  git_with_token(dir, args, authenticated.then_some("git-test-token")).await
}

async fn git_with_token(dir: &Path, args: &[&str], token: Option<&str>) -> Output {
  let mut command = tokio::process::Command::new("git");
  command
    .current_dir(dir)
    .env("GIT_TERMINAL_PROMPT", "0")
    .kill_on_drop(true);
  if let Some(token) = token {
    command.arg("-c").arg(format!(
      "http.extraHeader=Authorization: Basic {}",
      STANDARD.encode(format!("owner:{token}"))
    ));
  }
  tokio::time::timeout(
    std::time::Duration::from_secs(20),
    command.args(args).output(),
  )
  .await
  .unwrap()
  .unwrap()
}

async fn success(dir: &Path, args: &[&str]) -> String {
  let output = git(dir, args, true).await;
  assert!(
    output.status.success(),
    "git {args:?}: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

#[tokio::test]
async fn native_http_clone_fetch_push_and_protection_preserve_repository_integrity() {
  let temporary = tempfile::tempdir().unwrap();
  let root = temporary.path();
  let state = fixture(root).await;
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let url = format!("http://{}/team/demo.git", listener.local_addr().unwrap());
  let app = router(state.clone());
  let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
  let challenge = reqwest::get(format!("{url}/info/refs?service=git-upload-pack"))
    .await
    .unwrap();
  assert_eq!(challenge.status(), reqwest::StatusCode::UNAUTHORIZED);
  assert_eq!(
    challenge.headers()["www-authenticate"],
    "Basic realm=\"Gity\""
  );
  let denied = git(root, &["ls-remote", &url], false).await;
  assert!(
    !denied.status.success(),
    "private repository must reject anonymous reads"
  );
  success(root, &["clone", &url, "clone"]).await;
  let work = root.join("clone");
  assert_eq!(
    std::fs::read_to_string(work.join("README.md")).unwrap(),
    "base\n"
  );
  success(&work, &["config", "user.name", "Owner"]).await;
  success(&work, &["config", "user.email", "owner@example.test"]).await;
  let old = success(&work, &["rev-parse", "HEAD"]).await;
  std::fs::write(work.join("next.txt"), "accepted\n").unwrap();
  success(&work, &["add", "."]).await;
  success(&work, &["commit", "-m", "accepted push"]).await;
  success(&work, &["push", "origin", "main"]).await;
  let accepted = success(&work, &["rev-parse", "HEAD"]).await;
  assert_eq!(
    state
      .git
      .resolve_revision("20.git", "main", "")
      .await
      .unwrap(),
    accepted
  );
  success(&work, &["fetch", "origin"]).await;
  let project = state
    .db
    .one(
      "SELECT id FROM projects WHERE full_path=?",
      vec!["team/demo".into()],
    )
    .await
    .unwrap()
    .unwrap();
  insert(&state,"project_branch_protections",json!({"project_id":project["id"],"branch_name":"main","rule_type":"exact","push_access_level":"maintainer","merge_access_level":"maintainer","require_merge_request":0,"require_pipeline_success":0,"allow_force_push":0,"allow_delete":0})).await;
  success(&work, &["reset", "--hard", &old]).await;
  let forced = git(&work, &["push", "--force", "origin", "main"], true).await;
  assert!(!forced.status.success());
  assert!(
    String::from_utf8_lossy(&forced.stderr).contains("force push is not allowed"),
    "{}",
    String::from_utf8_lossy(&forced.stderr)
  );
  let deleted = git(&work, &["push", "origin", ":main"], true).await;
  assert!(!deleted.status.success());
  assert_eq!(
    state
      .git
      .resolve_revision("20.git", "main", "")
      .await
      .unwrap(),
    accepted,
    "rejected mutations must not change the branch"
  );
  server.abort();
}

#[tokio::test]
async fn api_scoped_project_tokens_clone_but_cannot_push_with_owner_privileges() {
  let temporary = tempfile::tempdir().unwrap();
  let root = temporary.path();
  let state = fixture(root).await;
  let project = state
    .db
    .one(
      "SELECT id FROM projects WHERE full_path='team/demo'",
      vec![],
    )
    .await
    .unwrap()
    .unwrap();
  let owner = state
    .db
    .one("SELECT id FROM users WHERE username='owner'", vec![])
    .await
    .unwrap()
    .unwrap();
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let url = format!("http://{}/team/demo.git", listener.local_addr().unwrap());
  let app = router(state.clone());
  let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
  for scope in ["read_api", "write_api"] {
    let raw = format!("git-{scope}-token");
    insert(
      &state,
      "project_access_tokens",
      json!({
          "project_id":project["id"],"created_by_user_id":owner["id"],
          "name":scope,"username":"ci-bot","kind":"access","scopes":scope,
          "token":format!("sha256:{}",gity_platform::auth::token_hash(&raw)),
          "expires_at":"2037-01-01T00:00:00Z","revoked_at":null,
      }),
    )
    .await;
    let cloned = git_with_token(root, &["clone", &url, scope], Some(&raw)).await;
    assert!(
      cloned.status.success(),
      "{scope}: {}",
      String::from_utf8_lossy(&cloned.stderr)
    );
    assert_eq!(
      std::fs::read_to_string(root.join(scope).join("README.md")).unwrap(),
      "base\n"
    );
    let pushed = git_with_token(
      &root.join(scope),
      &["push", "origin", "HEAD:refs/heads/token-branch"],
      Some(&raw),
    )
    .await;
    assert!(
      !pushed.status.success(),
      "API scope cannot grant repository writes"
    );
    assert!(
      String::from_utf8_lossy(&pushed.stderr).contains("403"),
      "{}",
      String::from_utf8_lossy(&pushed.stderr)
    );
  }
  assert!(
    state
      .git
      .resolve_revision("20.git", "token-branch", "")
      .await
      .is_err()
  );
  server.abort();
}

#[path = "git_http/namespace.rs"]
mod namespace;
