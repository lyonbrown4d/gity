use crate::{GatewaySettings, http_proxy, ssh_proxy};
use fluxdi::{
  Error as DiError, Injector, Provider, Shared,
  module::{Module, ModuleLifecycleFuture},
};
use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};
use tokio::{net::TcpListener, sync::Mutex, task::JoinHandle};
use tokio_util::sync::CancellationToken;

pub struct GatewayModule {
  settings: GatewaySettings,
}

#[derive(Clone, Default)]
pub struct GatewayHealth(Arc<AtomicBool>);

impl GatewayHealth {
  pub fn is_healthy(&self) -> bool {
    self.0.load(Ordering::Acquire)
  }
}

struct GatewayServer {
  settings: GatewaySettings,
  proxy: http_proxy::HttpProxy,
  health: GatewayHealth,
  stop: CancellationToken,
  http_task: Mutex<Option<JoinHandle<Result<(), std::io::Error>>>>,
  ssh_task: Mutex<Option<JoinHandle<Result<(), std::io::Error>>>>,
}

impl GatewayModule {
  pub fn new(settings: GatewaySettings) -> Result<Self, String> {
    settings.validate()?;
    Ok(Self { settings })
  }
}

impl Module for GatewayModule {
  fn configure(&self, injector: &Injector) -> Result<(), DiError> {
    let health = GatewayHealth(Arc::new(AtomicBool::new(false)));
    let proxy = http_proxy::HttpProxy::new(
      self.settings.main_http_origin.clone(),
      self.settings.request_timeout(),
      self.settings.max_http_requests,
      health.clone(),
    )
    .map_err(|error| DiError::module_lifecycle_failed("GatewayModule", "configure", &error))?
    .with_package_origin(self.settings.package_http_origin.clone())
    .with_public_scheme(self.settings.public_scheme.clone());
    let provider_health = health.clone();
    injector.try_provide(Provider::<GatewayHealth>::root(move |_| {
      Shared::new(provider_health.clone())
    }))?;
    let settings = self.settings.clone();
    let server_health = health.clone();
    injector.try_provide(Provider::<GatewayServer>::root(move |_| {
      Shared::new(GatewayServer {
        settings: settings.clone(),
        proxy: proxy.clone(),
        health: server_health.clone(),
        stop: CancellationToken::new(),
        http_task: Mutex::new(None),
        ssh_task: Mutex::new(None),
      })
    }))
  }

  fn on_start(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let server = resolve_server(&injector, "on_start")?;
      let http_listener = TcpListener::bind(server.settings.http_listen)
        .await
        .map_err(|error| lifecycle_error("on_start", &error.to_string()))?;
      let ssh_listener = TcpListener::bind(server.settings.ssh_listen)
        .await
        .map_err(|error| lifecycle_error("on_start", &error.to_string()))?;
      let http_stop = server.stop.clone();
      let http_health = server.health.clone();
      let http_router = http_proxy::router(server.proxy.clone());
      let http_task = tokio::spawn(async move {
        let result = axum::serve(http_listener, http_router)
          .with_graceful_shutdown(http_stop.cancelled_owned())
          .await;
        if result.is_err() {
          http_health.0.store(false, Ordering::Release);
        }
        result
      });
      let ssh_stop = server.stop.clone();
      let ssh_health = server.health.clone();
      let settings = server.settings.clone();
      let ssh_task = tokio::spawn(async move {
        let target = Arc::from(settings.main_ssh_address.clone());
        let max_sessions = settings.max_ssh_sessions;
        let connect_timeout = settings.ssh_connect_timeout();
        let session_timeout = settings.ssh_session_timeout();
        let shutdown_timeout = settings.shutdown_timeout();
        let result = ssh_proxy::serve(
          ssh_listener,
          target,
          max_sessions,
          connect_timeout,
          session_timeout,
          shutdown_timeout,
          ssh_stop,
        )
        .await;
        if result.is_err() {
          ssh_health.0.store(false, Ordering::Release);
        }
        result
      });
      *server.http_task.lock().await = Some(http_task);
      *server.ssh_task.lock().await = Some(ssh_task);
      server.health.0.store(true, Ordering::Release);
      Ok(())
    })
  }

  fn on_stop(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let server = resolve_server(&injector, "on_stop")?;
      server.health.0.store(false, Ordering::Release);
      server.stop.cancel();
      let shutdown_timeout = server.settings.shutdown_timeout();
      stop_task(&server.http_task, shutdown_timeout, "http").await?;
      stop_task(&server.ssh_task, shutdown_timeout, "ssh").await?;
      Ok(())
    })
  }
}

fn resolve_server(
  injector: &Injector,
  lifecycle: &'static str,
) -> Result<Shared<GatewayServer>, DiError> {
  injector.try_resolve::<GatewayServer>().map_err(|error| {
    DiError::module_lifecycle_failed("GatewayModule", lifecycle, &error.to_string())
  })
}

fn lifecycle_error(lifecycle: &'static str, error: &str) -> DiError {
  DiError::module_lifecycle_failed("GatewayModule", lifecycle, error)
}

async fn stop_task(
  task: &Mutex<Option<JoinHandle<Result<(), std::io::Error>>>>,
  timeout: std::time::Duration,
  name: &'static str,
) -> Result<(), DiError> {
  let Some(mut task) = task.lock().await.take() else {
    return Ok(());
  };
  let joined = match tokio::time::timeout(timeout, &mut task).await {
    Ok(joined) => joined,
    Err(_) => {
      task.abort();
      let _ = task.await;
      return Err(lifecycle_error(
        "on_stop",
        "gateway task shutdown timed out",
      ));
    }
  };
  joined
    .map_err(|error| lifecycle_error("on_stop", &error.to_string()))?
    .map_err(|error| DiError::module_lifecycle_failed("GatewayModule", name, &error.to_string()))
}
