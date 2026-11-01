use crate::{OciAuthorization, OciGateway, router};
use fluxdi::{
  Error as DiError, Injector, Provider, Shared,
  module::{Module, ModuleLifecycleFuture},
};
use std::{
  net::SocketAddr,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
};
use tokio::{net::TcpListener, sync::Mutex, task::JoinHandle};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct OciGatewayModule {
  address: SocketAddr,
  gateway: OciGateway,
}

#[derive(Clone)]
pub struct OciGatewayHealth(Arc<AtomicBool>);

impl OciGatewayHealth {
  pub fn is_healthy(&self) -> bool {
    self.0.load(Ordering::Acquire)
  }
}

impl OciGatewayModule {
  pub fn new(
    address: SocketAddr,
    settings: crate::OciSettings,
    authorization: Arc<dyn OciAuthorization>,
  ) -> Result<Self, String> {
    Self::new_with_mirror(address, settings, authorization, None)
  }

  pub fn new_with_mirror(
    address: SocketAddr,
    settings: crate::OciSettings,
    authorization: Arc<dyn OciAuthorization>,
    mirror_resolver: Option<Arc<dyn crate::MirrorResolver>>,
  ) -> Result<Self, String> {
    Ok(Self {
      address,
      gateway: OciGateway::new_with_mirror(settings, authorization, mirror_resolver)?,
    })
  }
}

struct OciGatewayServer {
  gateway: OciGateway,
  address: SocketAddr,
  health: OciGatewayHealth,
  stop: CancellationToken,
  task: Mutex<Option<JoinHandle<Result<(), std::io::Error>>>>,
}

impl Module for OciGatewayModule {
  fn configure(&self, injector: &Injector) -> Result<(), DiError> {
    let gateway = self.gateway.clone();
    let address = self.address;
    let health = OciGatewayHealth(Arc::new(AtomicBool::new(false)));
    let server_health = health.clone();
    injector.try_provide(Provider::<OciGatewayHealth>::root(move |_| {
      Shared::new(health.clone())
    }))?;
    injector.try_provide(Provider::<OciGatewayServer>::root(move |_| {
      Shared::new(OciGatewayServer {
        gateway: gateway.clone(),
        address,
        health: server_health.clone(),
        stop: CancellationToken::new(),
        task: Mutex::new(None),
      })
    }))
  }

  fn on_start(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let server = injector
        .try_resolve::<OciGatewayServer>()
        .map_err(|error| {
          DiError::module_lifecycle_failed("OciGatewayModule", "on_start", &error.to_string())
        })?;
      let listener = TcpListener::bind(server.address).await.map_err(|error| {
        DiError::module_lifecycle_failed("OciGatewayModule", "on_start", &error.to_string())
      })?;
      let stop = server.stop.clone();
      let gateway = server.gateway.clone();
      server.health.0.store(true, Ordering::Release);
      let health = server.health.clone();
      let task = tokio::spawn(async move {
        let result = axum::serve(listener, router(gateway))
          .with_graceful_shutdown(stop.cancelled_owned())
          .await;
        health.0.store(false, Ordering::Release);
        result
      });
      *server.task.lock().await = Some(task);
      Ok(())
    })
  }

  fn on_stop(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let server = injector
        .try_resolve::<OciGatewayServer>()
        .map_err(|error| {
          DiError::module_lifecycle_failed("OciGatewayModule", "on_stop", &error.to_string())
        })?;
      server.health.0.store(false, Ordering::Release);
      server.stop.cancel();
      if let Some(task) = server.task.lock().await.take() {
        task
          .await
          .map_err(|error| {
            DiError::module_lifecycle_failed("OciGatewayModule", "on_stop", &error.to_string())
          })?
          .map_err(|error| {
            DiError::module_lifecycle_failed("OciGatewayModule", "on_stop", &error.to_string())
          })?;
      }
      Ok(())
    })
  }
}
