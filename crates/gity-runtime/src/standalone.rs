use fluxdi::{
  Injector, Shared,
  module::{Module, ModuleLifecycleFuture},
};
use std::sync::Arc;

/// Composition root that lets FluxDI bootstrap component modules and manage
/// their start/stop hooks in import order. Each component remains responsible
/// for its own DI graph, health checks, and shutdown behavior.
pub struct StandaloneModule {
  components: Vec<Arc<dyn Fn() -> Box<dyn Module> + Send + Sync>>,
}

impl StandaloneModule {
  pub fn new(components: Vec<Arc<dyn Fn() -> Box<dyn Module> + Send + Sync>>) -> Self {
    Self { components }
  }
}

impl Module for StandaloneModule {
  fn imports(&self) -> Vec<Box<dyn Module>> {
    self.components.iter().map(|factory| factory()).collect()
  }

  fn on_start(&self, _injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async { Ok(()) })
  }
}
