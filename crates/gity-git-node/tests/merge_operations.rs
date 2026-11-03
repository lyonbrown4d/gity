mod support;
#[path = "merge_operations/transport.rs"]
mod transport;
use gity_git::{CreateFileCommitInput, MergeOperationInput, MergeOperationOutcome};
use gity_git_wire::GitOperation;

fn file(branch: &str, path: &str) -> CreateFileCommitInput {
  CreateFileCommitInput {
    expected_head_sha: None,
    update_existing: false,
    branch_name: branch.into(),
    file_path: path.into(),
    content: path.into(),
    message: path.into(),
    author_name: "Tester".into(),
    author_email: "test@example.test".into(),
  }
}

#[tokio::test]
async fn remote_merge_receipt_survives_node_restart_and_read_only_recovery() {
  let root = tempfile::tempdir().unwrap();
  let settings = support::settings("merge-node", root.path().join("repos"), "git".into());
  let node = support::start_node(settings.clone()).await;
  let git = support::remote(node.client.clone());
  git.init_bare("42.git", "main").await.unwrap();
  let target = git
    .create_file_commit("42.git", file("main", "base"))
    .await
    .unwrap();
  git
    .create_branch("42.git", "feature", "main")
    .await
    .unwrap();
  let source = git
    .create_file_commit("42.git", file("feature", "feature"))
    .await
    .unwrap();
  let input = MergeOperationInput {
    operation_key: "019a3955-1000-7000-8000-000000000010".into(),
    actor_user_id: 1,
    source_branch: "feature".into(),
    target_branch: "main".into(),
    source_sha: source,
    target_sha: target,
    message: "Remote merge".into(),
    author_name: "Tester".into(),
    author_email: "test@example.test".into(),
    timestamp: 1_800_000_000,
  };
  let request = serde_json::to_value(&input).unwrap();
  assert!(
    GitOperation::MergeOperation {
      input: request.clone()
    }
    .is_write()
  );
  assert!(
    GitOperation::AbortMergeOperation {
      input: request.clone(),
      reason: "operator".into(),
      actor_user_id: 1
    }
    .is_write()
  );
  assert!(!GitOperation::MergeOperationOutcome { input: request }.is_write());
  let result = git.merge_operation("42.git", input.clone()).await.unwrap();
  assert!(matches!(result, MergeOperationOutcome::Applied { .. }));
  git
    .create_file_commit("42.git", file("main", "later"))
    .await
    .unwrap();
  node.stop().await;
  let mut read_only = settings;
  read_only.read_only = true;
  let node = support::start_node(read_only).await;
  let recovered = support::remote(node.client.clone());
  assert_eq!(
    recovered
      .merge_operation_outcome("42.git", input.clone())
      .await
      .unwrap(),
    Some(result)
  );
  assert!(
    recovered
      .merge_operation("42.git", input.clone())
      .await
      .is_err()
  );
  assert!(
    recovered
      .abort_merge_operation("42.git", input, "operator".into(), 1)
      .await
      .is_err()
  );
  node.stop().await;
}
