mod support;

use gity_git::CreateFileCommitInput;
use gity_git_wire::GitOperation;
use std::{fs, os::unix::fs::PermissionsExt};
use support::{remote, settings, start_node};

#[tokio::test]
async fn invalid_bundle_keeps_a_resumable_initialization_marker() {
  let source_dir = tempfile::tempdir().unwrap();
  let target_dir = tempfile::tempdir().unwrap();
  let script = source_dir.path().join("broken-git");
  fs::write(&script, "#!/bin/sh\nif [ \"$1\" = bundle ] && [ \"$2\" = create ]; then printf broken; exit 0; fi\nexec git \"$@\"\n").unwrap();
  fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
  let source = start_node(settings("source", source_dir.path().join("repos"), script)).await;
  let target = start_node(settings(
    "target",
    target_dir.path().join("repos"),
    "git".into(),
  ))
  .await;
  remote(source.client.clone())
    .init_bare("42.git", "main")
    .await
    .unwrap();
  source
    .client
    .execute(
      "42.git",
      GitOperation::CreateFileCommit {
        input: serde_json::to_value(CreateFileCommitInput {
          branch_name: "main".into(),
          file_path: "one.txt".into(),
          content: "one".into(),
          message: "one".into(),
          author_name: "Replica Test".into(),
          author_email: "replica@example.org".into(),
        })
        .unwrap(),
      },
    )
    .await
    .unwrap();
  assert!(
    target
      .client
      .execute(
        "42.git",
        GitOperation::SyncReplica {
          source: source.client.endpoint().clone(),
          source_epoch: 1
        },
      )
      .await
      .is_err()
  );
  assert!(target_dir.path().join("repos/42.git").is_dir());
  let state: serde_json::Value = serde_json::from_slice(
    &fs::read(target_dir.path().join("repos/.gity-node-state/42.json")).unwrap(),
  )
  .unwrap();
  assert_eq!(state["role"], "initializing_replica");
  source.stop().await;
  target.stop().await;
}
