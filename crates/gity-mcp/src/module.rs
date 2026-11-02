use crate::{McpConfig, McpService};
use fluxdi::module::Module;
use fluxdi::{Injector, Provider, Shared, static_module};
use gity_api_contracts::ApplicationApiHandle;

fn make_service(
  backend: Shared<ApplicationApiHandle>,
  config: Shared<McpConfig>,
) -> Result<Shared<McpService>, fluxdi::Error> {
  McpService::new(backend.0.clone(), (*config).clone())
    .map(Shared::new)
    .map_err(|error| fluxdi::Error::factory_failed("McpService", &error.to_string()))
}

static_module! {
  pub struct McpModule {
    #[scope(root)]
    mcp_service: McpService = make_service(backend, config);
    external backend: ApplicationApiHandle;
    external config: McpConfig;
  }
}

impl Module for McpModule {
  fn configure(&self, injector: &Injector) -> Result<(), fluxdi::Error> {
    let service = self.mcp_service()?;
    injector.provide(Provider::<McpService>::root(move |_| service.clone()));
    Ok(())
  }
}
