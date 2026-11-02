//! FluxDI lifecycle for HTTP, SSH and background application services.
mod supervise;
use crate::AppState;
use fluxdi::{
  Error, Injector, Provider, Shared,
  module::{Module, ModuleLifecycleFuture},
};
use tokio::{
  sync::{Mutex, watch},
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy)]
pub struct PlatformOptions {
  pub http: bool,
  pub ssh: bool,
  pub scheduler: bool,
  pub maintenance: bool,
  pub migrate: bool,
}
impl Default for PlatformOptions {
  fn default() -> Self {
    Self {
      http: true,
      ssh: true,
      scheduler: true,
      maintenance: true,
      migrate: false,
    }
  }
}

#[derive(Clone)]
pub struct PlatformModule {
  state: AppState,
  options: PlatformOptions,
  failure: watch::Sender<Option<String>>,
}
impl PlatformModule {
  pub fn new(state: AppState, options: PlatformOptions) -> Self {
    let (failure, _) = watch::channel(None);
    Self {
      state,
      options,
      failure,
    }
  }
  pub fn subscribe(&self) -> watch::Receiver<Option<String>> {
    self.failure.subscribe()
  }
}
struct PlatformRuntime {
  state: AppState,
  stop: CancellationToken,
  task: Mutex<Option<JoinHandle<anyhow::Result<()>>>>,
  failure: watch::Sender<Option<String>>,
}
impl Module for PlatformModule {
  fn imports(&self) -> Vec<Box<dyn Module>> {
    let imports = self
      .state
      .git_cluster
      .as_ref()
      .map(|module| vec![Box::new(module.clone()) as Box<dyn Module>])
      .unwrap_or_default();
    #[cfg(feature = "packages")]
    let imports = imports
      .into_iter()
      .chain([
        Box::new(gity_package_registry::PackageRegistryModule::embedded(
          crate::package_authority::state(&self.state),
        )) as Box<dyn Module>,
      ])
      .collect();
    imports
  }

  fn configure(&self, injector: &Injector) -> Result<(), Error> {
    let state = Shared::new(self.state.clone());
    injector.try_provide(Provider::<AppState>::root(move |_| state.clone()))?;
    let state = self.state.clone();
    let failure = self.failure.clone();
    injector.try_provide(Provider::<PlatformRuntime>::root(move |_| {
      Shared::new(PlatformRuntime {
        state: state.clone(),
        stop: CancellationToken::new(),
        task: Mutex::new(None),
        failure: failure.clone(),
      })
    }))
  }
  fn on_start(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    let options = self.options;
    Box::pin(async move {
      let runtime = injector.try_resolve::<PlatformRuntime>()?;
      if options.migrate {
        runtime
          .state
          .db
          .migrate()
          .await
          .map_err(|e| failure("on_start", e))?;
      }
      // Bind and validate every listener before creating background tasks.
      let http = if options.http {
        Some(
          tokio::net::TcpListener::bind(&runtime.state.settings.address)
            .await
            .map_err(|e| failure("on_start", e))?,
        )
      } else {
        None
      };
      let ssh = if options.ssh && runtime.state.settings.ssh.enabled {
        let config =
          crate::ssh::configuration(&runtime.state).map_err(|e| failure("on_start", e))?;
        let listener = tokio::net::TcpListener::bind(&runtime.state.settings.ssh.address)
          .await
          .map_err(|e| failure("on_start", e))?;
        Some((listener, config))
      } else {
        None
      };
      let running = runtime.clone();
      *runtime.task.lock().await = Some(tokio::spawn(async move {
        supervise::run(running, options, http, ssh).await
      }));
      Ok(())
    })
  }
  fn on_stop(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let runtime = injector.try_resolve::<PlatformRuntime>()?;
      runtime.stop.cancel();
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
  Error::module_lifecycle_failed("PlatformModule", phase, &error.to_string())
}

pub use gity_runtime::shutdown_signal;
