use crate::{AppState, auth_tests::request, features::test_support::fixture};
use axum::http::StatusCode;
use serde_json::{Value, json};

async fn prepare() -> (tempfile::TempDir, AppState) {
  let (dir, state) = fixture().await;
  state
    .db
    .insert(
      "organization_members",
      json!({"organization_id":10,"user_id":1,"role":"developer"})
        .as_object()
        .unwrap()
        .clone(),
    )
    .await
    .unwrap();
  (dir, state)
}
fn project(key: &str) -> Value {
  json!({"organization_id":"10","name":"Template Project","key":key,"description":"A useful project","default_branch":"trunk"})
}
async fn create(state: &AppState, body: Value) -> String {
  let (status, reply) = request(state, "POST", "/projects", "alice", body).await;
  assert_eq!(status, StatusCode::OK, "{reply}");
  format!("{}.git", reply["data"]["id"].as_str().unwrap())
}
#[tokio::test]
async fn readme_license_and_gitignore_are_one_initial_commit_on_selected_branch() {
  let (_dir, s) = prepare().await;
  let mut body = project("initialized");
  body["license_template"] = json!("mit");
  body["license_copyright_holder"] = json!("Example Contributors");
  body["gitignore_template"] = json!("rust");
  body["initialize_with_readme"] = json!(true);
  let repo = create(&s, body).await;
  let tree = s.git.list_tree(&repo, "trunk", "", "").await.unwrap();
  let paths = tree
    .as_array()
    .unwrap()
    .iter()
    .map(|row| row["path"].as_str().unwrap())
    .collect::<Vec<_>>();
  assert!(paths.contains(&"README.md"));
  assert!(paths.contains(&"LICENSE"));
  assert!(paths.contains(&".gitignore"));
  assert_eq!(paths.len(), 3);
  let readme = s
    .git
    .get_blob(&repo, "trunk", "", "README.md")
    .await
    .unwrap();
  assert!(
    readme["content"]
      .as_str()
      .unwrap()
      .contains("Template Project")
  );
  assert!(
    readme["content"]
      .as_str()
      .unwrap()
      .contains("A useful project")
  );
  let license = s.git.get_blob(&repo, "trunk", "", "LICENSE").await.unwrap();
  assert!(
    license["content"]
      .as_str()
      .unwrap()
      .contains("Example Contributors")
  );
  assert!(
    license["content"]
      .as_str()
      .unwrap()
      .contains("Permission is hereby granted")
  );
  let ignore = s
    .git
    .get_blob(&repo, "trunk", "", ".gitignore")
    .await
    .unwrap();
  assert!(ignore["content"].as_str().unwrap().contains("target"));
  assert_eq!(
    s.git
      .list_commits(&repo, "trunk", "", 20)
      .await
      .unwrap()
      .as_array()
      .unwrap()
      .len(),
    1
  );
  assert_eq!(
    s.git.resolve_revision(&repo, "HEAD", "").await.unwrap(),
    s.git.resolve_revision(&repo, "trunk", "").await.unwrap()
  );
}
#[tokio::test]
async fn omitted_options_keep_empty_repo_and_readme_is_independently_optional() {
  let (_dir, s) = prepare().await;
  let empty = create(&s, project("empty")).await;
  assert!(
    s.git
      .list_branches(&empty, "trunk")
      .await
      .unwrap()
      .as_array()
      .unwrap()
      .is_empty()
  );
  let mut body = project("readme-only");
  body["initialize_with_readme"] = json!(true);
  let readme = create(&s, body).await;
  let tree = s.git.list_tree(&readme, "trunk", "", "").await.unwrap();
  assert_eq!(tree.as_array().unwrap().len(), 1);
  assert_eq!(tree[0]["path"], "README.md");
  let mut body = project("license-only");
  body["license_template"] = json!("mit");
  let licensed = create(&s, body).await;
  let tree = s.git.list_tree(&licensed, "trunk", "", "").await.unwrap();
  assert_eq!(tree.as_array().unwrap().len(), 1);
  assert_eq!(tree[0]["path"], "LICENSE");
  let license = s
    .git
    .get_blob(&licensed, "trunk", "", "LICENSE")
    .await
    .unwrap();
  assert!(
    license["content"].as_str().unwrap().contains("Team"),
    "holder defaults to owning organization"
  );
}
#[tokio::test]
async fn invalid_options_and_unauthorized_creation_have_no_project_side_effects() {
  let (_dir, s) = prepare().await;
  let count = s.projects.list_after(0, 100).await.unwrap().len();
  for (key, value) in [
    ("license_template", json!("../../secret")),
    ("license_template", json!(false)),
    ("gitignore_template", json!("unknown")),
    ("initialize_with_readme", json!("true")),
    ("license_copyright_holder", json!("Bad\nHolder")),
  ] {
    let mut body = project("invalid");
    body["license_template"] = json!("mit");
    body[key] = value;
    let (status, reply) = request(&s, "POST", "/projects", "alice", body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{key}: {reply}");
  }
  let (status, _) = request(&s, "POST", "/projects", "outsider", project("denied")).await;
  assert_eq!(status, StatusCode::FORBIDDEN);
  let (status, _) = request(&s, "POST", "/projects", "", project("anonymous")).await;
  assert_eq!(status, StatusCode::UNAUTHORIZED);
  assert_eq!(s.projects.list_after(0, 100).await.unwrap().len(), count);
}
#[tokio::test]
async fn failed_git_initialization_compensates_project_and_membership() {
  let (dir, mut s) = prepare().await;
  let blocker = dir.path().join("not-a-directory");
  std::fs::write(&blocker, "blocked").unwrap();
  s.git = gity_git::GitService::new(blocker, s.settings.git_bin.clone());
  let count = s.projects.list_after(0, 100).await.unwrap().len();
  let mut body = project("storage-failure");
  body["initialize_with_readme"] = json!(true);
  let (status, reply) = request(&s, "POST", "/projects", "alice", body).await;
  assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{reply}");
  assert_eq!(s.projects.list_after(0, 100).await.unwrap().len(), count);
}
#[tokio::test]
async fn public_license_catalogue_matches_embedded_templates() {
  let (_dir, s) = fixture().await;
  let (status, reply) = request(&s, "GET", "/repository-templates/licenses", "", Value::Null).await;
  assert_eq!(status, StatusCode::OK, "{reply}");
  let templates = reply["data"].as_array().unwrap();
  assert_eq!(templates.len(), 8);
  assert!(
    templates
      .iter()
      .any(|item| item["id"] == "mit" && item["spdx_id"] == "MIT")
  );
  assert!(templates.iter().any(|item| item["id"] == "gpl-3.0-only"));
  assert!(templates.iter().all(|item| item.get("content").is_none()));
}

#[tokio::test]
async fn public_gitignore_catalogue_exposes_official_candidates() {
  let (_dir, s) = fixture().await;
  let (status, reply) = request(
    &s,
    "GET",
    "/repository-templates/gitignores",
    "",
    Value::Null,
  )
  .await;
  assert_eq!(status, StatusCode::OK, "{reply}");
  let templates = reply["data"].as_array().unwrap();
  assert_eq!(templates.len(), 13);
  for id in [
    "rust",
    "node",
    "python",
    "go",
    "java",
    "cpp",
    "visual-studio",
    "dart",
    "swift",
    "kotlin",
    "ruby",
    "unity",
    "android",
  ] {
    assert!(
      templates.iter().any(|item| item["id"] == id),
      "missing {id}"
    );
  }
}
