use fluxdi::module::Module;
use fluxdi::{Injector, Provider, Shared, static_module};
use gity_core::OAuthConfig;
use sea_orm::DatabaseConnection;

use crate::OAuthService;

fn oauth_service(
  connection: Shared<DatabaseConnection>,
  config: Shared<OAuthConfig>,
  issuer: Shared<String>,
) -> Result<Shared<OAuthService>, fluxdi::Error> {
  let service = OAuthService::new((*connection).clone(), (*config).clone(), (*issuer).clone())
    .map_err(|error| fluxdi::Error::factory_failed("OAuthService", &error.to_string()))?;
  Ok(Shared::new(service))
}

static_module! {
  pub struct OAuthModule {
    #[scope(root)]
    service: OAuthService = oauth_service(connection, config, issuer);
    external connection: DatabaseConnection;
    external config: OAuthConfig;
    external issuer: String;
  }
}

impl Module for OAuthModule {
  fn configure(&self, injector: &Injector) -> Result<(), fluxdi::Error> {
    let service = self.service()?;
    injector.provide(Provider::<OAuthService>::root(move |_| service.clone()));
    Ok(())
  }
}
