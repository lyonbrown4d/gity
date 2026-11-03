use gity_core::{Db, GitClusterConfig, GitStorageNodeConfig};
use gity_git::{CreateFileCommitInput, GitService};
use gity_git_cluster::{GitCluster, ReplicaStatus};
use gity_git_node::{GitNodeService, GitNodeSettings};
use gity_git_wire::{GitNodeRouter, wire::git_storage_server::GitStorageServer};
use std::{path::Path, sync::Arc};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;

const TOKEN: &str = "cluster-functional-peer-test-credential";
async fn node(root: &Path, id: &str) -> (GitStorageNodeConfig, CancellationToken) {
  tokio::fs::create_dir_all(root).await.unwrap();
  let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let address = listener.local_addr().unwrap();
  let service = GitNodeService::new(GitNodeSettings {
    id: id.into(),
    address,
    repo_root: root.into(),
    git_bin: "git".into(),
    hook_executable: "/usr/bin/true".into(),
    token: TOKEN.into(),
    read_only: false,
    tls_certificate: None,
    tls_key: None,
    client_ca: None,
    replica_max_bytes: 128 * 1024 * 1024,
    replica_timeout_secs: 30,
  })
  .unwrap();
  let stop = CancellationToken::new();
  let signal = stop.clone();
  tokio::spawn(async move {
    tonic::transport::Server::builder()
      .add_service(GitStorageServer::new(service))
      .serve_with_incoming_shutdown(TcpListenerStream::new(listener), signal.cancelled_owned())
      .await
      .unwrap();
  });
  (
    GitStorageNodeConfig {
      id: id.into(),
      endpoint: format!("http://{address}"),
      token: TOKEN.into(),
      token_file: None,
      ca_file: None,
      client_certificate_file: None,
      client_key_file: None,
    },
    stop,
  )
}
fn input(content: &str) -> CreateFileCommitInput {
  CreateFileCommitInput {
    expected_head_sha: None,
    update_existing: false,
    branch_name: "main".into(),
    file_path: format!("{content}.txt"),
    content: content.into(),
    message: content.into(),
    author_name: "tester".into(),
    author_email: "tester@example.test".into(),
  }
}

#[tokio::test]
async fn current_replica_routing_fenced_promotion_and_uncertain_write_repair() {
  let dir = tempfile::tempdir().unwrap();
  let (a, stop_a) = node(&dir.path().join("a"), "a").await;
  let (b, stop_b) = node(&dir.path().join("b"), "b").await;
  let db = Db::connect(&format!(
    "sqlite://{}?mode=rwc",
    dir.path().join("db.sqlite").display()
  ))
  .await
  .unwrap();
  db.migrate().await.unwrap();
  let cluster = Arc::new(
    GitCluster::new(
      db.connection.clone(),
      GitClusterConfig {
        nodes: vec![a, b],
        replicas: 2,
        replication_interval_seconds: 5,
      },
    )
    .unwrap(),
  );
  let git = GitService::with_remote(cluster.clone());
  git.init_bare("42.git", "main").await.unwrap();
  git
    .create_file_commit("42.git", input("first"))
    .await
    .unwrap();
  let row = cluster.inspect(42).await.unwrap();
  let original = row.primary_node.clone();
  let target = row.replicas[0].node.clone();
  assert_eq!(row.pending_writes, 0);
  assert_eq!(row.readable_nodes().len(), 1);
  assert_eq!(cluster.replicate_tick().await.unwrap(), 1);
  let row = cluster.inspect(42).await.unwrap();
  assert_eq!(row.replicas[0].status, ReplicaStatus::Ready);
  assert_eq!(row.readable_nodes().len(), 2);
  git
    .create_file_commit("42.git", input("second"))
    .await
    .unwrap();
  assert_eq!(cluster.inspect(42).await.unwrap().readable_nodes().len(), 1);
  cluster.replicate_tick().await.unwrap();
  let promoted = cluster.promote(42, &target).await.unwrap();
  assert_eq!(promoted.primary_node, target);
  assert_eq!(promoted.epoch, 2);
  assert_eq!(
    git
      .get_blob("42.git", "main", "main", "second.txt")
      .await
      .unwrap()["content"],
    "second"
  );
  git
    .create_file_commit("42.git", input("third"))
    .await
    .unwrap();
  cluster.replicate_tick().await.unwrap();
  let pending = cluster.begin_write("42.git").await.unwrap();
  cluster
    .write_finished("42.git", &pending, false)
    .await
    .unwrap();
  assert_eq!(cluster.inspect(42).await.unwrap().readable_nodes().len(), 1);
  let repaired = cluster.promote(42, &target).await.unwrap();
  assert_eq!(repaired.epoch, 3);
  assert_eq!(repaired.pending_writes, 0);
  cluster.replicate_tick().await.unwrap();
  assert_eq!(
    git
      .get_blob("42.git", "main", "main", "third.txt")
      .await
      .unwrap()["content"],
    "third"
  );
  assert_ne!(original, target);
  stop_a.cancel();
  stop_b.cancel();
}
