mod support;

use gity_git::CreateFileCommitInput;
use gity_git_wire::GitOperation;
use std::fs;
use support::{remote, settings, start_node};

fn write_marker(root: &std::path::Path, role: &str, epoch: u64, branch: Option<&str>) {
  let state = root.join(".gity-node-state");
  fs::create_dir_all(&state).unwrap();
  fs::write(
    state.join("42.json"),
    serde_json::json!({
      "role": role,
      "epoch": epoch,
      "initial_branch": branch,
    })
    .to_string(),
  )
  .unwrap();
}

#[tokio::test]
async fn primary_initialization_resumes_owned_partial_directory_after_restart() {
  let temp = tempfile::tempdir().unwrap();
  let root = temp.path().join("repos");
  fs::create_dir_all(root.join("42.git")).unwrap();
  write_marker(&root, "initializing_primary", 1, Some("main"));
  let node = start_node(settings("primary", root, "git".into())).await;

  assert!(
    node
      .client
      .execute("42.git", GitOperation::ReferenceSnapshot)
      .await
      .is_err()
  );
  assert!(
    node
      .client
      .execute("42.git", GitOperation::FenceRepository { next_epoch: 2 })
      .await
      .is_err()
  );
  assert!(
    node
      .client
      .clone()
      .with_epoch(1)
      .execute("42.git", GitOperation::ActivatePrimary { epoch: 1 })
      .await
      .is_err()
  );
  node
    .client
    .execute(
      "42.git",
      GitOperation::InitBare {
        initial_branch: "main".into(),
      },
    )
    .await
    .unwrap();
  node
    .client
    .execute("42.git", GitOperation::ReferenceSnapshot)
    .await
    .unwrap();
  node.stop().await;
}

#[tokio::test]
async fn replica_initialization_resumes_owned_partial_directory_after_restart() {
  let source_temp = tempfile::tempdir().unwrap();
  let source = start_node(settings(
    "source",
    source_temp.path().join("repos"),
    "git".into(),
  ))
  .await;
  let source_git = remote(source.client.clone());
  source_git.init_bare("42.git", "main").await.unwrap();
  source
    .client
    .execute(
      "42.git",
      GitOperation::CreateFileCommit {
        input: serde_json::to_value(CreateFileCommitInput {
          expected_head_sha: None,
          update_existing: false,
          branch_name: "main".into(),
          file_path: "resume.txt".into(),
          content: "resumed".into(),
          message: "resume source".into(),
          author_name: "Replica Test".into(),
          author_email: "replica@example.org".into(),
        })
        .unwrap(),
      },
    )
    .await
    .unwrap();

  let target_temp = tempfile::tempdir().unwrap();
  let target_root = target_temp.path().join("repos");
  fs::create_dir_all(target_root.join("42.git")).unwrap();
  write_marker(&target_root, "initializing_replica", 1, None);
  let target = start_node(settings("target", target_root, "git".into())).await;
  target
    .client
    .execute(
      "42.git",
      GitOperation::SyncReplica {
        source: source.client.endpoint().clone(),
        source_epoch: 1,
      },
    )
    .await
    .unwrap();
  let snapshot: gity_git_wire::ReferenceSnapshot = serde_json::from_value(
    target
      .client
      .execute("42.git", GitOperation::ReferenceSnapshot)
      .await
      .unwrap(),
  )
  .unwrap();
  assert!(snapshot.refs.contains_key("refs/heads/main"));
  target.stop().await;
  source.stop().await;
}

#[tokio::test]
async fn initialization_refuses_an_unmarked_existing_directory() {
  let temp = tempfile::tempdir().unwrap();
  let root = temp.path().join("repos");
  fs::create_dir_all(root.join("42.git")).unwrap();
  let node = start_node(settings("unmanaged", root, "git".into())).await;
  assert!(
    node
      .client
      .execute(
        "42.git",
        GitOperation::InitBare {
          initial_branch: "main".into(),
        },
      )
      .await
      .is_err()
  );
  assert!(
    node
      .client
      .execute(
        "42.git",
        GitOperation::SyncReplica {
          source: gity_git_wire::NodeEndpoint {
            id: "unmanaged-source".into(),
            uri: "http://127.0.0.1:1".into(),
            token: support::TOKEN.into(),
            ca_certificate: None,
            client_certificate: None,
            client_key: None,
          },
          source_epoch: 1,
        },
      )
      .await
      .is_err()
  );
  node.stop().await;
}
