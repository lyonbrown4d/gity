mod support;

use gity_git::CreateFileCommitInput;
use gity_git_wire::GitOperation;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};
use support::{remote, settings, start_node};

fn commit(path: &str, message: &str) -> GitOperation {
  GitOperation::CreateFileCommit {
    input: serde_json::to_value(CreateFileCommitInput {
      expected_head_sha: None,
      update_existing: false,
      branch_name: "main".into(),
      file_path: path.into(),
      content: message.into(),
      message: message.into(),
      author_name: "Replica Test".into(),
      author_email: "replica@example.org".into(),
    })
    .unwrap(),
  }
}

#[tokio::test]
async fn sync_fence_restart_and_explicit_promotion_enforce_epochs() {
  let source_dir = tempfile::tempdir().unwrap();
  let target_dir = tempfile::tempdir().unwrap();
  let source = start_node(settings(
    "source",
    source_dir.path().join("repos"),
    "git".into(),
  ))
  .await;
  let target = start_node(settings(
    "target",
    target_dir.path().join("repos"),
    "git".into(),
  ))
  .await;
  let source_git = remote(source.client.clone());
  source_git.init_bare("42.git", "main").await.unwrap();
  source
    .client
    .execute("42.git", commit("one.txt", "one"))
    .await
    .unwrap();
  source_git
    .create_branch("42.git", "release", "main")
    .await
    .unwrap();
  source_git.create_tag("42.git", "v1", "main").await.unwrap();

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
  let target_git = remote(target.client.clone());
  assert_eq!(
    target_git
      .list_branches("42.git", "main")
      .await
      .unwrap()
      .as_array()
      .unwrap()
      .len(),
    2
  );

  source
    .client
    .execute("42.git", commit("two.txt", "two"))
    .await
    .unwrap();
  source_git.delete_branch("42.git", "release").await.unwrap();
  source_git.delete_tag("42.git", "v1").await.unwrap();
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
  assert_eq!(
    target_git
      .list_branches("42.git", "main")
      .await
      .unwrap()
      .as_array()
      .unwrap()
      .len(),
    1
  );
  assert!(
    target_git
      .list_tags("42.git")
      .await
      .unwrap()
      .as_array()
      .unwrap()
      .is_empty()
  );

  source
    .client
    .execute("42.git", GitOperation::FenceRepository { next_epoch: 2 })
    .await
    .unwrap();
  source
    .client
    .execute("42.git", GitOperation::FenceRepository { next_epoch: 2 })
    .await
    .unwrap();
  assert!(
    source
      .client
      .execute("42.git", commit("stale.txt", "stale"))
      .await
      .is_err()
  );
  let source_settings = source.settings.clone();
  source.stop().await;
  let restarted = start_node(source_settings).await;
  assert!(
    restarted
      .client
      .execute("42.git", commit("stale.txt", "stale"))
      .await
      .is_err()
  );
  restarted
    .client
    .clone()
    .with_epoch(2)
    .execute("42.git", GitOperation::ReferenceSnapshot)
    .await
    .unwrap();

  let promoted = target.client.clone().with_epoch(2);
  promoted
    .execute("42.git", GitOperation::ActivatePrimary { epoch: 2 })
    .await
    .unwrap();
  promoted
    .execute("42.git", commit("primary.txt", "primary"))
    .await
    .unwrap();
  assert!(
    target
      .client
      .execute("42.git", commit("old.txt", "old"))
      .await
      .is_err()
  );
  restarted.stop().await;
  target.stop().await;
}

#[tokio::test]
async fn dropping_bundle_consumer_reaps_git_process() {
  let source_dir = tempfile::tempdir().unwrap();
  let script = source_dir.path().join("slow-git");
  let marker = source_dir.path().join("pid");
  let body = format!(
    "#!/bin/sh\nif [ \"$1\" = bundle ] && [ \"$2\" = create ]; then echo $$ > '{}'; printf chunk; exec sleep 30; fi\nexec git \"$@\"\n",
    marker.display()
  );
  fs::write(&script, body).unwrap();
  fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
  let source = start_node(settings("source", source_dir.path().join("repos"), script)).await;
  remote(source.client.clone())
    .init_bare("42.git", "main")
    .await
    .unwrap();
  source
    .client
    .execute("42.git", commit("one.txt", "one"))
    .await
    .unwrap();

  let (send, incoming) = tokio::sync::mpsc::channel(2);
  send
    .send(gity_git_wire::wire::TransferFrame {
      payload: Some(gity_git_wire::wire::transfer_frame::Payload::OpenJson(
        serde_json::to_vec(&gity_git_wire::TransferOpen {
          epoch: 1,
          repository: gity_git_wire::RepositoryKey::try_from("42.git".to_owned()).unwrap(),
          kind: gity_git_wire::TransferKind::Bundle,
        })
        .unwrap(),
      )),
    })
    .await
    .unwrap();
  send
    .send(gity_git_wire::wire::TransferFrame {
      payload: Some(gity_git_wire::wire::transfer_frame::Payload::InputFinished(
        true,
      )),
    })
    .await
    .unwrap();
  drop(send);
  let request = source
    .client
    .authenticate(tokio_stream::wrappers::ReceiverStream::new(incoming))
    .unwrap();
  let mut response = source
    .client
    .client()
    .transfer(request)
    .await
    .unwrap()
    .into_inner();
  assert!(matches!(
    response.message().await.unwrap().unwrap().payload,
    Some(gity_git_wire::wire::transfer_frame::Payload::SnapshotJson(
      _
    ))
  ));
  let _chunk = response.message().await.unwrap().unwrap();
  let pid = tokio::time::timeout(Duration::from_secs(2), async {
    loop {
      if let Ok(value) = tokio::fs::read_to_string(&marker).await
        && let Ok(pid) = value.trim().parse::<u32>()
      {
        break pid;
      }
      tokio::time::sleep(Duration::from_millis(10)).await;
    }
  })
  .await
  .unwrap();
  drop(response);
  tokio::time::timeout(Duration::from_secs(3), async {
    while PathBuf::from(format!("/proc/{pid}")).exists() {
      tokio::time::sleep(Duration::from_millis(20)).await;
    }
  })
  .await
  .unwrap();
  source.stop().await;
}
