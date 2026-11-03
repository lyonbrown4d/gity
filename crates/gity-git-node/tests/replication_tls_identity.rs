use gity_git::{CreateFileCommitInput, GitService};
use gity_git_node::{GitNodeService, GitNodeSettings};
use gity_git_wire::{
  FixedNodeRouter, GitOperation, NodeClient, NodeEndpoint,
  wire::git_storage_server::GitStorageServer,
};
use std::{
  fs,
  net::SocketAddr,
  path::{Path, PathBuf},
  process::Command,
  sync::Arc,
};
use tokio::{net::TcpListener, task::JoinHandle};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;

const TOKEN: &str = "node-internal-test-credential-32-bytes";

struct Identity {
  certificate: PathBuf,
  key: PathBuf,
}

struct RunningNode {
  address: SocketAddr,
  client: NodeClient,
  stop: CancellationToken,
  task: JoinHandle<()>,
}

impl RunningNode {
  async fn stop(self) {
    self.stop.cancel();
    self.task.await.unwrap();
  }
}

fn openssl(arguments: &[&std::ffi::OsStr]) {
  let output = Command::new("openssl").args(arguments).output().unwrap();
  assert!(
    output.status.success(),
    "openssl failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

fn create_ca(directory: &Path, name: &str) -> Identity {
  let identity = Identity {
    certificate: directory.join(format!("{name}-ca.crt")),
    key: directory.join(format!("{name}-ca.key")),
  };
  openssl(&[
    "req".as_ref(),
    "-x509".as_ref(),
    "-newkey".as_ref(),
    "rsa:2048".as_ref(),
    "-nodes".as_ref(),
    "-keyout".as_ref(),
    identity.key.as_os_str(),
    "-out".as_ref(),
    identity.certificate.as_os_str(),
    "-days".as_ref(),
    "2".as_ref(),
    "-subj".as_ref(),
    format!("/CN={name}-test-ca").as_ref(),
  ]);
  identity
}

fn create_identity(directory: &Path, name: &str, ca: &Identity) -> Identity {
  let identity = Identity {
    certificate: directory.join(format!("{name}.crt")),
    key: directory.join(format!("{name}.key")),
  };
  let request = directory.join(format!("{name}.csr"));
  let extensions = directory.join(format!("{name}.ext"));
  fs::write(
    &extensions,
    "basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth,clientAuth\nsubjectAltName=IP:127.0.0.1\n",
  )
  .unwrap();
  openssl(&[
    "req".as_ref(),
    "-newkey".as_ref(),
    "rsa:2048".as_ref(),
    "-nodes".as_ref(),
    "-keyout".as_ref(),
    identity.key.as_os_str(),
    "-out".as_ref(),
    request.as_os_str(),
    "-subj".as_ref(),
    format!("/CN={name}").as_ref(),
  ]);
  openssl(&[
    "x509".as_ref(),
    "-req".as_ref(),
    "-in".as_ref(),
    request.as_os_str(),
    "-CA".as_ref(),
    ca.certificate.as_os_str(),
    "-CAkey".as_ref(),
    ca.key.as_os_str(),
    "-CAcreateserial".as_ref(),
    "-out".as_ref(),
    identity.certificate.as_os_str(),
    "-days".as_ref(),
    "2".as_ref(),
    "-extfile".as_ref(),
    extensions.as_os_str(),
  ]);
  identity
}

fn client(id: &str, address: SocketAddr, server_ca: &Identity, identity: &Identity) -> NodeClient {
  NodeClient::new(NodeEndpoint {
    id: id.into(),
    uri: format!("https://{address}"),
    token: TOKEN.into(),
    ca_certificate: Some(fs::read_to_string(&server_ca.certificate).unwrap()),
    client_certificate: Some(fs::read_to_string(&identity.certificate).unwrap()),
    client_key: Some(fs::read_to_string(&identity.key).unwrap()),
  })
  .unwrap()
}

async fn start_node(
  id: &str,
  root: PathBuf,
  server_identity: &Identity,
  client_ca: &Identity,
  caller_identity: &Identity,
  server_ca: &Identity,
) -> RunningNode {
  tokio::fs::create_dir_all(&root).await.unwrap();
  let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let address = listener.local_addr().unwrap();
  let settings = GitNodeSettings {
    id: id.into(),
    address,
    repo_root: root,
    git_bin: "git".into(),
    hook_executable: "/usr/bin/true".into(),
    token: TOKEN.into(),
    read_only: false,
    replica_max_bytes: 1024 * 1024,
    replica_timeout_secs: 10,
    tls_certificate: Some(server_identity.certificate.clone()),
    tls_key: Some(server_identity.key.clone()),
    client_ca: Some(client_ca.certificate.clone()),
  };
  let service = GitNodeService::new(settings).unwrap();
  let server_certificate = tokio::fs::read(&server_identity.certificate).await.unwrap();
  let server_key = tokio::fs::read(&server_identity.key).await.unwrap();
  let trusted_client_ca = tokio::fs::read(&client_ca.certificate).await.unwrap();
  let stop = CancellationToken::new();
  let shutdown = stop.clone();
  let task = tokio::spawn(async move {
    tonic::transport::Server::builder()
      .tls_config(
        tonic::transport::ServerTlsConfig::new()
          .identity(tonic::transport::Identity::from_pem(
            server_certificate,
            server_key,
          ))
          .client_ca_root(tonic::transport::Certificate::from_pem(trusted_client_ca)),
      )
      .unwrap()
      .add_service(GitStorageServer::new(service))
      .serve_with_incoming_shutdown(TcpListenerStream::new(listener), shutdown.cancelled_owned())
      .await
      .unwrap();
  });
  RunningNode {
    address,
    client: client(id, address, server_ca, caller_identity),
    stop,
    task,
  }
}

#[tokio::test]
async fn replica_uses_its_node_identity_and_never_forwards_api_identity() {
  let temp = tempfile::tempdir().unwrap();
  let node_ca = create_ca(temp.path(), "node");
  let api_ca = create_ca(temp.path(), "api");
  let source_identity = create_identity(temp.path(), "source", &node_ca);
  let target_identity = create_identity(temp.path(), "target", &node_ca);
  let api_identity = create_identity(temp.path(), "api-client", &api_ca);
  let source = start_node(
    "git-1",
    temp.path().join("source-repos"),
    &source_identity,
    &node_ca,
    &target_identity,
    &node_ca,
  )
  .await;
  let target = start_node(
    "git-2",
    temp.path().join("target-repos"),
    &target_identity,
    &api_ca,
    &api_identity,
    &node_ca,
  )
  .await;

  let source_as_peer = client("git-1", source.address, &node_ca, &target_identity);
  let source_git = GitService::with_remote(Arc::new(FixedNodeRouter(source_as_peer.clone())));
  source_git.init_bare("42.git", "main").await.unwrap();
  source_as_peer
    .execute(
      "42.git",
      GitOperation::CreateFileCommit {
        input: serde_json::to_value(CreateFileCommitInput {
          expected_head_sha: None,
          update_existing: false,
          branch_name: "main".into(),
          file_path: "hello.txt".into(),
          content: "replicated".into(),
          message: "replication fixture".into(),
          author_name: "Test".into(),
          author_email: "test@example.org".into(),
        })
        .unwrap(),
      },
    )
    .await
    .unwrap();
  let expected = source_as_peer
    .execute("42.git", GitOperation::ReferenceSnapshot)
    .await
    .unwrap();

  let api_view_of_source = client("git-1", source.address, &node_ca, &api_identity);
  let endpoint = api_view_of_source.replication_endpoint();
  assert!(endpoint.client_certificate.is_none());
  assert!(endpoint.client_key.is_none());
  target
    .client
    .execute(
      "42.git",
      GitOperation::SyncReplica {
        source: endpoint,
        source_epoch: 1,
      },
    )
    .await
    .unwrap();
  let actual = target
    .client
    .execute("42.git", GitOperation::ReferenceSnapshot)
    .await
    .unwrap();
  assert_eq!(actual, expected);

  source.stop().await;
  target.stop().await;
}
