use crate::{PackageState, routes};
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
pub struct PackageRegistryModule {
  address: Option<SocketAddr>,
  state: PackageState,
}

#[derive(Clone)]
pub struct PackageRegistryHealth(Arc<AtomicBool>);

impl PackageRegistryHealth {
  pub fn is_healthy(&self) -> bool {
    self.0.load(Ordering::Acquire)
  }
}

impl PackageRegistryModule {
  pub fn new(address: SocketAddr, state: PackageState) -> Self {
    Self {
      address: Some(address),
      state,
    }
  }

  pub fn embedded(state: PackageState) -> Self {
    Self {
      address: None,
      state,
    }
  }
}

struct PackageRegistryServer {
  address: Option<SocketAddr>,
  state: PackageState,
  health: PackageRegistryHealth,
  stop: CancellationToken,
  task: Mutex<Option<JoinHandle<Result<(), std::io::Error>>>>,
}

impl Module for PackageRegistryModule {
  fn configure(&self, injector: &Injector) -> Result<(), DiError> {
    let health = PackageRegistryHealth(Arc::new(AtomicBool::new(false)));
    let server_health = health.clone();
    let address = self.address;
    let state = self.state.clone();
    injector.try_provide(Provider::<PackageRegistryHealth>::root(move |_| {
      Shared::new(health.clone())
    }))?;
    injector.try_provide(Provider::<PackageRegistryServer>::root(move |_| {
      Shared::new(PackageRegistryServer {
        address,
        state: state.clone(),
        health: server_health.clone(),
        stop: CancellationToken::new(),
        task: Mutex::new(None),
      })
    }))
  }

  fn on_start(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let server = injector
        .try_resolve::<PackageRegistryServer>()
        .map_err(|error| {
          DiError::module_lifecycle_failed("PackageRegistryModule", "on_start", &error.to_string())
        })?;
      let Some(address) = server.address else {
        server.health.0.store(true, Ordering::Release);
        return Ok(());
      };
      let listener = TcpListener::bind(address).await.map_err(|error| {
        DiError::module_lifecycle_failed("PackageRegistryModule", "on_start", &error.to_string())
      })?;
      let stop = server.stop.clone();
      let health = server.health.clone();
      let app = axum::Router::new()
        .nest("/api/v1", routes())
        .with_state(server.state.clone())
        .layer(axum::middleware::from_fn(gity_observability::instrument))
        .layer(axum::middleware::from_fn(gity_core::locale::request));
      let task = tokio::spawn(async move {
        let result = axum::serve(listener, app)
          .with_graceful_shutdown(stop.cancelled_owned())
          .await;
        health.0.store(false, Ordering::Release);
        result
      });
      server.health.0.store(true, Ordering::Release);
      *server.task.lock().await = Some(task);
      Ok(())
    })
  }

  fn on_stop(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let server = injector
        .try_resolve::<PackageRegistryServer>()
        .map_err(|error| {
          DiError::module_lifecycle_failed("PackageRegistryModule", "on_stop", &error.to_string())
        })?;
      server.health.0.store(false, Ordering::Release);
      server.stop.cancel();
      if let Some(task) = server.task.lock().await.take() {
        task
          .await
          .map_err(|error| {
            DiError::module_lifecycle_failed("PackageRegistryModule", "on_stop", &error.to_string())
          })?
          .map_err(|error| {
            DiError::module_lifecycle_failed("PackageRegistryModule", "on_stop", &error.to_string())
          })?;
      }
      Ok(())
    })
  }
}
