use gity_git_node::{GitNodeService, GitNodeSettings};
use gity_git_wire::{
  FixedNodeRouter, NodeClient, NodeEndpoint, wire::git_storage_server::GitStorageServer,
};
use std::{path::PathBuf, sync::Arc};
use tokio::{net::TcpListener, task::JoinHandle};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;

pub const TOKEN: &str = "node-internal-test-credential-32-bytes";

pub fn settings(id: &str, root: PathBuf, git_bin: PathBuf) -> GitNodeSettings {
  GitNodeSettings {
    id: id.to_owned(),
    address: "127.0.0.1:0".parse().unwrap(),
    repo_root: root,
    git_bin,
    hook_executable: PathBuf::from("/usr/bin/true"),
    token: TOKEN.to_owned(),
    read_only: false,
    replica_max_bytes: 4 * 1024 * 1024,
    replica_timeout_secs: 10,
    tls_certificate: None,
    tls_key: None,
    client_ca: None,
  }
}

pub struct RunningNode {
  #[allow(dead_code)]
  pub settings: GitNodeSettings,
  pub client: NodeClient,
  stop: CancellationToken,
  server: JoinHandle<()>,
}

impl RunningNode {
  pub async fn stop(self) {
    self.stop.cancel();
    self.server.await.unwrap();
  }
}

pub async fn start_node(mut settings: GitNodeSettings) -> RunningNode {
  tokio::fs::create_dir_all(&settings.repo_root)
    .await
    .unwrap();
  let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let address = listener.local_addr().unwrap();
  settings.address = address;
  let service = GitNodeService::new(settings.clone()).unwrap();
  let stop = CancellationToken::new();
  let signal = stop.clone();
  let server = tokio::spawn(async move {
    tonic::transport::Server::builder()
      .add_service(GitStorageServer::new(service))
      .serve_with_incoming_shutdown(TcpListenerStream::new(listener), signal.cancelled_owned())
      .await
      .unwrap();
  });
  RunningNode {
    client: NodeClient::new(NodeEndpoint {
      id: settings.id.clone(),
      uri: format!("http://{address}"),
      token: TOKEN.to_owned(),
      ca_certificate: None,
      client_certificate: None,
      client_key: None,
    })
    .unwrap(),
    settings,
    stop,
    server,
  }
}

pub fn remote(client: NodeClient) -> gity_git::GitService {
  gity_git::GitService::with_remote(Arc::new(FixedNodeRouter(client)))
}
