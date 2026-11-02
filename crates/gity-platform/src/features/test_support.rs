use super::*;
use axum::{
  body::{Body, to_bytes},
  http::{Request, StatusCode},
};
use tower::ServiceExt;

pub(crate) async fn fixture() -> (tempfile::TempDir, AppState) {
  let dir = tempfile::tempdir().unwrap();
  let mut settings = gity_core::Settings::load();
  settings.repo_root = dir.path().join("repos");
  settings.storage_root = dir.path().join("storage");
  settings.search.code_index_root = dir.path().join("code-index");
  tokio::fs::create_dir_all(&settings.repo_root)
    .await
    .unwrap();
  let db = disposable_database().await;
  db.migrate().await.unwrap();
  for (id, name) in [(1, "alice"), (2, "bob"), (3, "outsider")] {
    db.insert("users",fields(json!({"id":id,"username":name,"display_name":name,"email":format!("{name}@test.local"),"is_super_admin":0}))).await.unwrap();
    db.insert(
            "user_access_tokens",
            fields(json!({"user_id":id,"name":"test-personal-token","token":format!("sha256:{}",crate::auth::token_hash(name))})),
        )
        .await
        .unwrap();
  }
  db.insert(
    "organizations",
    fields(
      json!({"id":10,"name":"Team","path_key":"team","full_path":"team","visibility":"private"}),
    ),
  )
  .await
  .unwrap();
  for p in [20, 30] {
    db.insert("projects",fields(json!({"id":p,"organization_id":10,"name":format!("Project {p}"),"path_key":format!("p{p}"),"full_path":format!("team/p{p}"),"visibility":"private","default_branch":"main","status":"active"}))).await.unwrap();
    for user in [1, 2] {
      db.insert(
        "project_members",
        fields(json!({"project_id":p,"user_id":user,"role":"maintainer"})),
      )
      .await
      .unwrap();
    }
  }
  let state = AppState {
    oauth: fluxdi::Shared::new(
      gity_identity::OAuthService::new(
        db.connection.clone(),
        settings.oauth.clone(),
        settings.base_url.clone(),
      )
      .unwrap(),
    ),
    ci: gity_ci::CiModule::new(fluxdi::Shared::new(db.connection.clone()))
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
    git_cluster: None,
    collaboration: gity_collaboration::CollaborationModule::new(fluxdi::Shared::new(
      db.connection.clone(),
    ))
    .service()
    .unwrap(),
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
      gity_git::GitService::new(&settings.repo_root, &settings.git_bin),
      gity_search::Limits::default(),
    )
    .unwrap(),
    code_index: gity_code_intel::Store::new(dir.path().join("code-index")).unwrap(),
    git: gity_git::GitService::new(&settings.repo_root, &settings.git_bin),
    storage: gity_core::Storage::local(&settings.storage_root).unwrap(),
    settings: std::sync::Arc::new(settings),
    db,
  };
  (dir, state)
}
pub(crate) async fn request(
  s: &AppState,
  method: &str,
  path: &str,
  user: &str,
  body: Value,
) -> (StatusCode, Value) {
  let router = routes().with_state(s.clone());
  let mut req = Request::builder()
    .method(method)
    .uri(path)
    .header("content-type", "application/json");
  if !user.is_empty() {
    req = req.header("authorization", format!("Bearer {user}"));
  }
  let response = router
    .oneshot(req.body(Body::from(body.to_string())).unwrap())
    .await
    .unwrap();
  let status = response.status();
  let data = to_bytes(response.into_body(), 20 * 1024 * 1024)
    .await
    .unwrap();
  (status, serde_json::from_slice(&data).unwrap_or(Value::Null))
}

pub(crate) async fn disposable_database() -> gity_core::Db {
  let Ok(url) = std::env::var("GITY_TEST_DATABASE_URL") else {
    return gity_core::Db::connect("sqlite::memory:").await.unwrap();
  };
  let _ = tracing_subscriber::fmt()
    .with_env_filter("gity_core=error")
    .with_test_writer()
    .try_init();
  let mut target = reqwest::Url::parse(&url).unwrap();
  assert!(
    matches!(target.host_str(), Some("127.0.0.1" | "localhost")),
    "test databases must be disposable localhost instances"
  );
  let name = format!("gity_test_{}", uuid::Uuid::new_v4().simple());
  let admin = gity_core::Db::connect(&url).await.unwrap();
  admin
    .execute(&format!("CREATE DATABASE {name}"), vec![])
    .await
    .unwrap();
  target.set_path(&name);
  gity_core::Db::connect(target.as_str()).await.unwrap()
}
