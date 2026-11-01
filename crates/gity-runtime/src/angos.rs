mod supervisor;
use fluxdi::{
  Error as DiError, Injector, Provider, Shared,
  module::{Module, ModuleLifecycleFuture},
};
use std::{
  collections::BTreeMap,
  path::PathBuf,
  process::Stdio,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  time::Duration,
};
use supervisor::supervise;
use tokio::{
  process::Command,
  sync::Mutex,
  time::{Instant, sleep},
};

#[derive(Clone)]
pub struct AngosConfig {
  pub executable: PathBuf,
  pub config_files: Vec<PathBuf>,
  pub health_url: String,
  pub startup_timeout: Duration,
  pub restart_limit: u32,
  pub shutdown_timeout: Duration,
  pub environment: BTreeMap<String, String>,
}

impl AngosConfig {
  pub fn validate(&self) -> Result<(), String> {
    if self.executable.as_os_str().is_empty()
      || self.config_files.is_empty()
      || self
        .config_files
        .iter()
        .any(|path| path.as_os_str().is_empty())
    {
      return Err("Angos executable and config files are required".into());
    }
    let health = reqwest::Url::parse(&self.health_url).map_err(|error| error.to_string())?;
    if !matches!(health.scheme(), "http" | "https") || health.host_str().is_none() {
      return Err("Angos health URL must be absolute HTTP(S)".into());
    }
    if self.startup_timeout.is_zero() {
      return Err("Angos startup timeout must be positive".into());
    }
    if self.shutdown_timeout.is_zero() {
      return Err("Angos shutdown timeout must be positive".into());
    }
    Ok(())
  }
}

/// Process handle owned by the runtime module; it is registered in FluxDI for
/// other modules' readiness checks and application-level shutdown.
pub struct AngosProcess {
  task: Mutex<Option<tokio::task::JoinHandle<()>>>,
  stop: tokio_util::sync::CancellationToken,
  healthy: Arc<AtomicBool>,
  failed: Arc<AtomicBool>,
}

impl AngosProcess {
  pub fn has_failed(&self) -> bool {
    self.failed.load(Ordering::Acquire)
  }
  pub fn is_healthy(&self) -> bool {
    self.healthy.load(Ordering::Acquire)
  }
}

impl AngosProcess {
  async fn start(&self, config: &AngosConfig) -> anyhow::Result<()> {
    let mut command = command(config);
    let mut child = command.spawn()?;
    let deadline = Instant::now() + config.startup_timeout;
    loop {
      if let Some(status) = child.try_wait()? {
        anyhow::bail!("Angos exited before becoming healthy: {status}");
      }
      if probe(&config.health_url).await {
        let config = config.clone();
        let stop = self.stop.clone();
        self.healthy.store(true, Ordering::Release);
        let healthy = self.healthy.clone();
        let failed = self.failed.clone();
        let stopped = stop.clone();
        let task = tokio::spawn(async move {
          supervise(config, child, stop, healthy).await;
          if !stopped.is_cancelled() {
            failed.store(true, Ordering::Release);
          }
        });
        *self.task.lock().await = Some(task);
        tracing::info!("Angos is healthy");
        return Ok(());
      }
      if Instant::now() >= deadline {
        let _ = child.kill().await;
        anyhow::bail!("Angos health check timed out");
      }
      sleep(Duration::from_millis(250)).await;
    }
  }

  async fn stop(&self) -> anyhow::Result<()> {
    self.stop.cancel();
    self.healthy.store(false, Ordering::Release);
    if let Some(task) = self.task.lock().await.take() {
      task.await?;
    }
    Ok(())
  }
}

/// FluxDI module that starts the pinned/preinstalled Angos executable only
/// behind Gity's private OCI gateway and owns its process lifetime.
#[derive(Clone)]
pub struct AngosModule {
  config: AngosConfig,
}

impl AngosModule {
  pub fn new(config: AngosConfig) -> Result<Self, String> {
    config.validate()?;
    Ok(Self { config })
  }
}

impl Module for AngosModule {
  fn configure(&self, injector: &Injector) -> Result<(), DiError> {
    injector.try_provide(Provider::<AngosProcess>::root(|_| {
      Shared::new(AngosProcess {
        task: Mutex::new(None),
        stop: tokio_util::sync::CancellationToken::new(),
        healthy: Arc::new(AtomicBool::new(false)),
        failed: Arc::new(AtomicBool::new(false)),
      })
    }))
  }

  fn on_start(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    let config = self.config.clone();
    Box::pin(async move {
      let process = injector.try_resolve::<AngosProcess>().map_err(|error| {
        DiError::module_lifecycle_failed("AngosModule", "on_start", &error.to_string())
      })?;
      process.start(&config).await.map_err(|error| {
        DiError::module_lifecycle_failed("AngosModule", "on_start", &error.to_string())
      })
    })
  }

  fn on_stop(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let process = injector.try_resolve::<AngosProcess>().map_err(|error| {
        DiError::module_lifecycle_failed("AngosModule", "on_stop", &error.to_string())
      })?;
      process.stop().await.map_err(|error| {
        DiError::module_lifecycle_failed("AngosModule", "on_stop", &error.to_string())
      })
    })
  }
}

fn command(config: &AngosConfig) -> Command {
  let mut command = Command::new(&config.executable);
  for file in &config.config_files {
    command.arg("-c").arg(file);
  }
  command
    .arg("server")
    .envs(&config.environment)
    .stdin(Stdio::null())
    .kill_on_drop(true);
  command
}

async fn probe(url: &str) -> bool {
  tokio::time::timeout(Duration::from_secs(2), reqwest::get(url))
    .await
    .ok()
    .and_then(Result::ok)
    .is_some_and(|response| response.status().is_success())
}
