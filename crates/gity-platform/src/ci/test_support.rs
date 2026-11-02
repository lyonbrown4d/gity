use super::*;
use axum::{
  body::{Body, to_bytes},
  http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;

pub(super) async fn fixture() -> (tempfile::TempDir, AppState, Router, String, i64) {
  let dir = tempfile::tempdir().unwrap();
  let root = dir.path().join("repos");
  std::fs::create_dir(&root).unwrap();
  let db = crate::features::test_support::disposable_database().await;
  db.migrate().await.unwrap();
  let user = db
    .insert(
      "users",
      json!({
              "username":"owner",
              "display_name":"Owner",
              "email":"owner@example.test",
              "is_super_admin":1
          }
      )
      .as_object()
      .unwrap()
      .clone(),
    )
    .await
    .unwrap();
  let org = db
    .insert(
      "organizations",
      json!({
              "name":"Team",
              "path_key":"team",
              "full_path":"team",
              "description":"",
              "visibility":"private"
          }
      )
      .as_object()
      .unwrap()
      .clone(),
    )
    .await
    .unwrap();
  let project = db
    .insert(
      "projects",
      json!({
              "organization_id":id(&org),
              "name":"Demo",
              "path_key":"demo",
              "full_path":"team/demo",
              "description":"",
              "visibility":"private",
              "default_branch":"main",
              "status":"active"
          }
      )
      .as_object()
      .unwrap()
      .clone(),
    )
    .await
    .unwrap();
  let token = "test-access-token".to_owned();
  db.execute(
    "INSERT INTO auth_sessions(token_hash,user_id,kind,expires_at,family) VALUES (?,?,?,?,?)",
    vec![
      crate::auth::token_hash(&token).into(),
      id(&user).into(),
      "access".into(),
      worker::BLOCKED.into(),
      "test".into(),
    ],
  )
  .await
  .unwrap();
  let git = gity_git::GitService::new(&root, "git");
  git
    .init_bare(&crate::repository::repo(&project), "main")
    .await
    .unwrap();
  let yaml = "stages: [build, test]\njobs:\n  build: {stage: build, script: 'echo build', tags: [linux]}\n  test: {script: 'echo test', tags: [linux]}\n";
  git
    .create_file_commit(
      &crate::repository::repo(&project),
      gity_git::CreateFileCommitInput {
        branch_name: "main".into(),
        file_path: ".gity-ci.yml".into(),
        content: yaml.into(),
        message: "Add YAML CI".into(),
        author_name: "Owner".into(),
        author_email: "owner@example.test".into(),
      },
    )
    .await
    .unwrap();
  let storage_config = gity_core::StorageConfig {
    driver: "local".into(),
    root: dir.path().join("storage"),
    s3_bucket: String::new(),
    s3_region: "us-east-1".into(),
    s3_endpoint: String::new(),
    s3_access_key: String::new(),
    s3_secret_key: String::new(),
    s3_use_path_style: true,
  };
  let observability = gity_core::ObservabilityConfig {
    service_name: "ci-test".into(),
    log_format: "text".into(),
    log_level: "warn".into(),
    metrics_enabled: false,
    metrics_address: "127.0.0.1:0".into(),
    otlp_endpoint: String::new(),
  };
  let settings = gity_core::Settings {
    git_cluster: None,
    components: Default::default(),
    address: "127.0.0.1:0".into(),
    base_url: "http://localhost".into(),
    database_url: "sqlite::memory:".into(),
    database_node_id: 1,
    repo_root: root,
    git_bin: "git".into(),
    git_hook: None,
    ssh: gity_core::SshConfig {
      enabled: false,
      address: "127.0.0.1:0".into(),
      host_key: dir.path().join("ssh_host_ed25519"),
      max_connections: 8,
      operation_timeout_seconds: 30,
    },
    storage_root: dir.path().join("storage"),
    storage: storage_config,
    web_root: dir.path().join("web"),
    observability,
    search: gity_core::SearchConfig {
      index_enabled: true,
      index_root: dir.path().join("search"),
      code_index_root: dir.path().join("code-index"),
      refresh_interval_seconds: 300,
      max_files_per_project: 5000,
      max_file_size_bytes: 524288,
    },
    oauth: Default::default(),
    api: Default::default(),
    bootstrap: gity_core::BootstrapConfig::default(),
  };
  let storage = gity_core::Storage::local(&settings.storage_root).unwrap();
  let state = AppState {
    oauth: fluxdi::Shared::new(
      gity_identity::OAuthService::new(
        db.connection.clone(),
        settings.oauth.clone(),
        settings.base_url.clone(),
      )
      .unwrap(),
    ),
    collaboration: gity_collaboration::CollaborationModule::new(fluxdi::Shared::new(
      db.connection.clone(),
    ))
    .service()
    .unwrap(),
    projects: gity_projects::ProjectModule::new(fluxdi::Shared::new(db.connection.clone()))
      .service()
      .unwrap(),
    repository_access: gity_projects::ProjectModule::new(fluxdi::Shared::new(
      db.connection.clone(),
    ))
    .repository_access()
    .unwrap(),
    ci: gity_ci::CiModule::new(fluxdi::Shared::new(db.connection.clone()))
      .service()
      .unwrap(),
    git_cluster: None,
    identity: gity_identity::IdentityModule::new(fluxdi::Shared::new(db.connection.clone()))
      .repository()
      .unwrap(),
    artifacts: gity_artifacts::ArtifactModule::new(
      fluxdi::Shared::new(db.connection.clone()),
      fluxdi::Shared::new(gity_core::Storage::local(&settings.storage_root).unwrap()),
    )
    .service()
    .unwrap(),
    search: gity_search::SearchIndex::new(
      dir.path().join("search"),
      git.clone(),
      gity_search::Limits::default(),
    )
    .unwrap(),
    code_index: gity_code_intel::Store::new(dir.path().join("code-index")).unwrap(),
    db,
    git,
    settings: Arc::new(settings),
    storage,
  };
  let app = routes().with_state(state.clone());
  (dir, state, app, token, id(&project))
}
pub(super) async fn request(
  app: &Router,
  method: &str,
  path: &str,
  token: Option<&str>,
  body: Value,
) -> (StatusCode, Value) {
  let mut req = Request::builder()
    .method(method)
    .uri(path)
    .header("content-type", "application/json");
  if let Some(token) = token {
    req = req.header("authorization", format!("Bearer {token}"));
  }
  let response = app
    .clone()
    .oneshot(req.body(Body::from(body.to_string())).unwrap())
    .await
    .unwrap();
  let status = response.status();
  let bytes = to_bytes(response.into_body(), 100 * 1024 * 1024)
    .await
    .unwrap();
  let response: Value = serde_json::from_slice(&bytes)
    .unwrap_or_else(|_| panic!("invalid response: {}", String::from_utf8_lossy(&bytes)));
  (status, response)
}
pub(super) async fn post(app: &Router, path: &str, token: Option<&str>, body: Value) -> Value {
  let (status, response) = request(app, "POST", path, token, body).await;
  assert_eq!(status, StatusCode::OK, "{response}");
  response["data"].clone()
}
