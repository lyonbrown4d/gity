use super::*;

#[tokio::test]
async fn organization_key_rename_keeps_existing_git_history_cloneable_at_the_new_url() {
  let temporary = tempfile::tempdir().unwrap();
  let root = temporary.path();
  let state = fixture(root).await;
  let organization = state
    .db
    .one(
      "SELECT id FROM organizations WHERE full_path='team'",
      vec![],
    )
    .await
    .unwrap()
    .unwrap();
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let address = listener.local_addr().unwrap();
  let app = router(state.clone());
  let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
  let response = reqwest::Client::new()
    .patch(format!(
      "http://{address}/api/v1/orgs/{}",
      organization["id"]
    ))
    .bearer_auth("git-test-token")
    .json(&json!({"key":"renamed","name":"Renamed Team"}))
    .send()
    .await
    .unwrap();
  let status = response.status();
  let response: Value = response.json().await.unwrap();
  assert_eq!(status, reqwest::StatusCode::OK, "{response}");
  let old = git(
    root,
    &["ls-remote", &format!("http://{address}/team/demo.git")],
    true,
  )
  .await;
  assert!(!old.status.success());
  success(
    root,
    &[
      "clone",
      &format!("http://{address}/renamed/demo.git"),
      "renamed",
    ],
  )
  .await;
  assert_eq!(
    std::fs::read_to_string(root.join("renamed/README.md")).unwrap(),
    "base\n"
  );
  assert!(root.join("repos/20.git").is_dir());
  assert!(!root.join("repos/team/demo.git").exists());
  assert!(!root.join("repos/renamed/demo.git").exists());
  server.abort();
}
