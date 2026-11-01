use gity_git::{CreateFileCommitInput, GitService};
use gity_git_node::{GitNodeService, GitNodeSettings};
use gity_git_wire::{
  FixedNodeRouter, NodeClient, NodeEndpoint, wire::git_storage_server::GitStorageServer,
};
use std::{path::PathBuf, sync::Arc};
use tokio::{io::AsyncReadExt, net::TcpListener};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;

const TOKEN: &str = "node-internal-test-credential-32-bytes";

async fn fixture(
  read_only: bool,
) -> (tempfile::TempDir, GitService, NodeClient, CancellationToken) {
  let dir = tempfile::tempdir().unwrap();
  let root = dir.path().join("repos");
  tokio::fs::create_dir(&root).await.unwrap();
  let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let address = listener.local_addr().unwrap();
  let settings = GitNodeSettings {
    id: "node-a".into(),
    address,
    repo_root: root,
    git_bin: "git".into(),
    hook_executable: PathBuf::from("/usr/bin/true"),
    token: TOKEN.into(),
    read_only,
    replica_max_bytes: 1024 * 1024,
    replica_timeout_secs: 30,
    tls_certificate: None,
    tls_key: None,
    client_ca: None,
  };
  let service = GitNodeService::new(settings).unwrap();
  let stop = CancellationToken::new();
  let shutdown = stop.clone();
  tokio::spawn(async move {
    tonic::transport::Server::builder()
      .add_service(GitStorageServer::new(service))
      .serve_with_incoming_shutdown(TcpListenerStream::new(listener), shutdown.cancelled_owned())
      .await
      .unwrap();
  });
  let node = NodeClient::new(NodeEndpoint {
    id: "node-a".into(),
    uri: format!("http://{address}"),
    token: TOKEN.into(),
    ca_certificate: None,
    client_certificate: None,
    client_key: None,
  })
  .unwrap();
  let git = GitService::with_remote(Arc::new(FixedNodeRouter(node.clone())));
  (dir, git, node, stop)
}

#[tokio::test]
async fn remote_reads_mutations_and_streaming_archive_have_no_gateway_repository_files() {
  let (dir, git, node, stop) = fixture(false).await;
  assert_eq!(node.health().await.unwrap().node_id, "node-a");
  git.init_bare("42.git", "main").await.unwrap();
  assert!(git.repository_path("42.git").is_err());
  let content = "Rust Git node\n".repeat(20_000);
  let commit = git
    .create_file_commit(
      "42.git",
      CreateFileCommitInput {
        branch_name: "main".into(),
        file_path: "README.md".into(),
        content: content.clone(),
        message: "first remote commit".into(),
        author_name: "tester".into(),
        author_email: "test@example.test".into(),
      },
    )
    .await
    .unwrap();
  assert_eq!(
    git
      .resolve_revision("42.git", "main", "main")
      .await
      .unwrap(),
    commit
  );
  assert!(
    git
      .file_exists("42.git", "main", "main", "README.md")
      .await
      .unwrap()
  );
  assert_eq!(
    git
      .get_blob("42.git", "main", "main", "README.md")
      .await
      .unwrap()["content"],
    content
  );
  git.create_branch("42.git", "topic", "main").await.unwrap();
  git.create_tag("42.git", "v1", "main").await.unwrap();
  assert_eq!(
    git
      .list_branches("42.git", "main")
      .await
      .unwrap()
      .as_array()
      .unwrap()
      .len(),
    2
  );
  assert_eq!(
    git
      .list_tags("42.git")
      .await
      .unwrap()
      .as_array()
      .unwrap()
      .len(),
    1
  );
  git.delete_branch("42.git", "topic").await.unwrap();
  git.delete_tag("42.git", "v1").await.unwrap();
  let mut archive = git
    .spawn_archive("42.git", "main")
    .await
    .unwrap()
    .into_reader()
    .unwrap();
  let mut bytes = Vec::new();
  archive.read_to_end(&mut bytes).await.unwrap();
  assert!(bytes.starts_with(b"PK"));
  assert!(dir.path().join("repos/42.git/HEAD").exists());
  assert!(git.init_bare("../43.git", "main").await.is_err());
  stop.cancel();
}

#[tokio::test]
async fn read_only_node_rejects_mutations() {
  let (_dir, git, node, stop) = fixture(true).await;
  assert!(!node.health().await.unwrap().writable);
  assert!(git.init_bare("42.git", "main").await.is_err());
  stop.cancel();
}

#[tokio::test]
async fn nonlocal_connections_require_tls_and_endpoint_debug_redacts_credentials() {
  let endpoint = NodeEndpoint {
    id: "remote".into(),
    uri: "http://10.0.0.1:9101".into(),
    token: TOKEN.into(),
    ca_certificate: None,
    client_certificate: None,
    client_key: None,
  };
  assert!(!format!("{endpoint:?}").contains(TOKEN));
  assert!(NodeClient::new(endpoint).is_err());
}
