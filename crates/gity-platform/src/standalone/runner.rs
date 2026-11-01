//! A separate Runner process is enrolled using private project credentials.
use crate::AppState;
use fluxdi::{
  Error, Injector, Provider, Shared,
  module::{Module, ModuleLifecycleFuture},
};
use std::{
  process::Stdio,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
};
#[derive(Clone)]
pub struct RunnerHealth(Arc<AtomicBool>);
impl RunnerHealth {
  pub fn is_healthy(&self) -> bool {
    self.0.load(Ordering::Acquire)
  }
}
use tokio::{process::Command, sync::Mutex, task::JoinHandle};
use tokio_util::sync::CancellationToken;
#[derive(Clone)]
pub struct RunnerModule {
  state: AppState,
}
impl RunnerModule {
  pub fn new(state: AppState) -> Self {
    Self { state }
  }
}
struct Runtime {
  state: AppState,
  stop: CancellationToken,
  task: Mutex<Option<JoinHandle<anyhow::Result<()>>>>,
  health: RunnerHealth,
}
impl Module for RunnerModule {
  fn configure(&self, injector: &Injector) -> Result<(), Error> {
    let health = RunnerHealth(Arc::new(AtomicBool::new(false)));
    let registered_health = health.clone();
    injector.try_provide(Provider::<RunnerHealth>::root(move |_| {
      Shared::new(registered_health.clone())
    }))?;
    let state = self.state.clone();
    injector.try_provide(Provider::<Runtime>::root(move |_| {
      Shared::new(Runtime {
        state: state.clone(),
        health: health.clone(),
        stop: CancellationToken::new(),
        task: Mutex::new(None),
      })
    }))
  }
  fn on_start(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let runtime = injector.try_resolve::<Runtime>()?;
      let settings = &runtime.state.settings.components.runner;
      tokio::fs::create_dir_all(&settings.token_directory)
        .await
        .map_err(failure)?;
      let address: std::net::SocketAddr =
        runtime.state.settings.address.parse().map_err(failure)?;
      let mut command = Command::new(&settings.executable);
      command
        .args([
          "--server",
          &format!("http://127.0.0.1:{}", address.port()),
          "--executor",
          "container",
          "--token-directory",
        ])
        .arg(&settings.token_directory)
        .arg("--work-root")
        .arg(&settings.work_root)
        .arg("--container-bin")
        .arg(&settings.container_bin)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true);
      let mut child = command.spawn().map_err(failure)?;
      let stop = runtime.stop.clone();
      let health = runtime.health.clone();
      health.0.store(true, Ordering::Release);
      *runtime.task.lock().await = Some(tokio::spawn(async move {
        tokio::select! {status=child.wait()=>{health.0.store(false,Ordering::Release); anyhow::bail!("standalone Runner exited unexpectedly: {}",status?);},_=stop.cancelled()=>{
          #[cfg(unix)] if let Some(pid)=child.id(){unsafe{libc::kill(pid as i32,libc::SIGTERM);}}
          if tokio::time::timeout(std::time::Duration::from_secs(15),child.wait()).await.is_err(){child.kill().await?;child.wait().await?;}
        }}
        health.0.store(false, Ordering::Release);
        Ok(())
      }));
      Ok(())
    })
  }
  fn on_stop(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let runtime = injector.try_resolve::<Runtime>()?;
      runtime.stop.cancel();
      if let Some(task) = runtime.task.lock().await.take() {
        task.await.map_err(failure)?.map_err(failure)?;
      }
      Ok(())
    })
  }
}
fn failure(error: impl std::fmt::Display) -> Error {
  Error::module_lifecycle_failed("RunnerModule", "lifecycle", &error.to_string())
}
pub(crate) async fn enroll(
  state: &AppState,
  runner: i64,
  token: &str,
) -> Result<(), gity_core::AppError> {
  if state.settings.components.standalone && state.settings.components.runner.enabled {
    super::secrets::write_private(
      &state
        .settings
        .components
        .runner
        .token_directory
        .join(format!("{runner}.token")),
      token.as_bytes(),
    )
    .await
    .map_err(|_| gity_core::AppError::internal("failed to enroll standalone Runner"))?;
  }
  Ok(())
}

pub(crate) async fn revoke(state: &AppState, runner: i64) -> Result<(), gity_core::AppError> {
  if state.settings.components.standalone && state.settings.components.runner.enabled {
    let path = state
      .settings
      .components
      .runner
      .token_directory
      .join(format!("{runner}.token"));
    match tokio::fs::remove_file(path).await {
      Ok(()) => {}
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
      Err(_) => {
        return Err(gity_core::AppError::internal(
          "failed to remove standalone Runner credential",
        ));
      }
    }
  }
  Ok(())
}
