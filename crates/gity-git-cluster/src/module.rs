use crate::GitCluster;
use fluxdi::{
  Error, Injector, Provider, Shared,
  module::{Module, ModuleLifecycleFuture},
  static_module,
};
use gity_core::GitClusterConfig;
use sea_orm::DatabaseConnection;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle};
use tokio_util::sync::CancellationToken;

fn catalog(
  config: Shared<GitClusterConfig>,
  connection: Shared<DatabaseConnection>,
) -> Result<Shared<GitCluster>, Error> {
  GitCluster::new((*connection).clone(), (*config).clone())
    .map(Shared::new)
    .map_err(|error| Error::factory_failed("GitCluster", &error.to_string()))
}
static_module! {
  struct Bindings {
    #[scope(root)]
    catalog:GitCluster=catalog(config,connection);
    external config:GitClusterConfig;
    external connection:DatabaseConnection;
  }
}
#[derive(Clone)]
pub struct GitClusterModule {
  bindings: Arc<Bindings>,
}
impl GitClusterModule {
  pub fn new(config: GitClusterConfig, connection: DatabaseConnection) -> Result<Self, Error> {
    let bindings = Arc::new(Bindings::new(Shared::new(config), Shared::new(connection)));
    bindings.catalog()?;
    Ok(Self { bindings })
  }
  pub fn catalog(&self) -> Result<Shared<GitCluster>, Error> {
    self.bindings.catalog()
  }
}
struct Worker {
  stop: CancellationToken,
  task: Mutex<Option<JoinHandle<()>>>,
}
impl Module for GitClusterModule {
  fn configure(&self, injector: &Injector) -> Result<(), Error> {
    let catalog = self.catalog()?;
    injector.try_provide(Provider::<GitCluster>::root(move |_| catalog.clone()))?;
    injector.try_provide(Provider::<Worker>::root(move |_| {
      Shared::new(Worker {
        stop: CancellationToken::new(),
        task: Mutex::new(None),
      })
    }))
  }
  fn on_start(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let worker = injector.try_resolve::<Worker>()?;
      let catalog = injector.try_resolve::<GitCluster>()?;
      let stop = worker.stop.clone();
      *worker.task.lock().await = Some(tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(
          catalog.config.replication_interval_seconds,
        ));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
          tokio::select! {_=stop.cancelled()=>break,_=tick.tick()=>{
            tokio::select! {_=stop.cancelled()=>break,result=catalog.replicate_tick()=>{if let Err(error)=result{tracing::warn!(error=%error,"Git replica reconciliation tick failed");}}}
          }}
        }
      }));
      Ok(())
    })
  }
  fn on_stop(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let worker = injector.try_resolve::<Worker>()?;
      worker.stop.cancel();
      if let Some(task) = worker.task.lock().await.take() {
        task.await.map_err(|error| {
          Error::module_lifecycle_failed("GitClusterModule", "on_stop", &error.to_string())
        })?;
      }
      Ok(())
    })
  }
}
