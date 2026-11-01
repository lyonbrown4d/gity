use crate::GitNodeService;
use fluxdi::{
  Error, Injector, Provider, Shared,
  module::{Module, ModuleLifecycleFuture},
};
use gity_git_wire::wire::git_storage_server::GitStorageServer;
use std::{net::SocketAddr, path::PathBuf};
use tokio::{net::TcpListener, sync::Mutex, task::JoinHandle};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};

#[derive(Clone)]
pub struct GitNodeSettings {
  pub id: String,
  pub address: SocketAddr,
  pub repo_root: PathBuf,
  pub git_bin: PathBuf,
  pub hook_executable: PathBuf,
  pub token: String,
  pub read_only: bool,
  pub replica_max_bytes: u64,
  pub replica_timeout_secs: u64,
  pub tls_certificate: Option<PathBuf>,
  pub tls_key: Option<PathBuf>,
  pub client_ca: Option<PathBuf>,
}
impl GitNodeSettings {
  pub fn validate(&self) -> anyhow::Result<()> {
    anyhow::ensure!(!self.id.is_empty(), "Git node identifier is required");
    anyhow::ensure!(
      self.token.len() >= 32,
      "Git node internal credential must contain at least 32 bytes"
    );
    let tls = self.tls_certificate.is_some() && self.tls_key.is_some() && self.client_ca.is_some();
    anyhow::ensure!(
      self.address.ip().is_loopback() || tls,
      "non-loopback Git storage listeners require mutual TLS"
    );
    anyhow::ensure!(
      self.tls_certificate.is_none() && self.tls_key.is_none() && self.client_ca.is_none() || tls,
      "all mutual TLS paths must be configured together"
    );
    anyhow::ensure!(
      self.replica_max_bytes > 0,
      "replica byte limit must be positive"
    );
    anyhow::ensure!(
      self.replica_timeout_secs > 0,
      "replica timeout must be positive"
    );
    Ok(())
  }
}
#[derive(Clone)]
pub struct GitNodeModule {
  settings: GitNodeSettings,
}
impl GitNodeModule {
  pub fn new(settings: GitNodeSettings) -> anyhow::Result<Self> {
    settings.validate()?;
    Ok(Self { settings })
  }
}
struct GitNodeRuntime {
  service: GitNodeService,
  stop: CancellationToken,
  task: Mutex<Option<JoinHandle<Result<(), tonic::transport::Error>>>>,
}
impl Module for GitNodeModule {
  fn configure(&self, injector: &Injector) -> Result<(), Error> {
    let service =
      GitNodeService::new(self.settings.clone()).map_err(|error| failure("configure", error))?;
    injector.try_provide(Provider::<GitNodeRuntime>::root(move |_| {
      Shared::new(GitNodeRuntime {
        service: service.clone(),
        stop: CancellationToken::new(),
        task: Mutex::new(None),
      })
    }))
  }
  fn on_start(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let runtime = injector.try_resolve::<GitNodeRuntime>()?;
      let settings = &runtime.service.settings;
      tokio::fs::create_dir_all(&settings.repo_root)
        .await
        .map_err(|e| failure("on_start", e))?;
      let listener = TcpListener::bind(settings.address)
        .await
        .map_err(|e| failure("on_start", e))?;
      gity_git_wire::configure_tls_crypto();
      let mut server = Server::builder();
      if let (Some(cert), Some(key), Some(ca)) = (
        &settings.tls_certificate,
        &settings.tls_key,
        &settings.client_ca,
      ) {
        let cert = tokio::fs::read(cert)
          .await
          .map_err(|e| failure("on_start", e))?;
        let key = tokio::fs::read(key)
          .await
          .map_err(|e| failure("on_start", e))?;
        let ca = tokio::fs::read(ca)
          .await
          .map_err(|e| failure("on_start", e))?;
        server = server
          .tls_config(
            ServerTlsConfig::new()
              .identity(Identity::from_pem(cert, key))
              .client_ca_root(Certificate::from_pem(ca)),
          )
          .map_err(|e| failure("on_start", e))?;
      }
      let stop = runtime.stop.clone();
      let service = GitStorageServer::new(runtime.service.clone())
        .max_decoding_message_size(8 * 1024 * 1024)
        .max_encoding_message_size(8 * 1024 * 1024);
      *runtime.task.lock().await = Some(tokio::spawn(async move {
        server
          .add_service(service)
          .serve_with_incoming_shutdown(TcpListenerStream::new(listener), stop.cancelled_owned())
          .await
      }));
      Ok(())
    })
  }
  fn on_stop(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let runtime = injector.try_resolve::<GitNodeRuntime>()?;
      runtime.stop.cancel();
      runtime.service.stop_workers().await;
      if let Some(task) = runtime.task.lock().await.take() {
        task
          .await
          .map_err(|e| failure("on_stop", e))?
          .map_err(|e| failure("on_stop", e))?;
      }
      Ok(())
    })
  }
}
fn failure(phase: &str, error: impl std::fmt::Display) -> Error {
  Error::module_lifecycle_failed("GitNodeModule", phase, &error.to_string())
}
