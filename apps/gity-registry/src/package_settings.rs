use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PackageSettings {
  pub enabled: bool,
  pub address: SocketAddr,
  pub database_dsn: String,
  pub database_node_id: u16,
  pub storage: gity_core::StorageConfig,
}
impl Default for PackageSettings {
  fn default() -> Self {
    Self {
      enabled: true,
      address: "0.0.0.0:8082".parse().unwrap(),
      database_dsn: "sqlite://data/packages.db?mode=rwc".into(),
      database_node_id: 5,
      storage: Default::default(),
    }
  }
}
impl PackageSettings {
  pub async fn module(
    self,
    endpoint: gity_git_wire::NodeEndpoint,
    timeout: std::time::Duration,
  ) -> anyhow::Result<Option<gity_package_registry::PackageRegistryModule>> {
    if !self.enabled {
      return Ok(None);
    }
    anyhow::ensure!(
      self.database_node_id < 1024,
      "package database node ID must be 0–1023"
    );
    gity_ids::install(gity_ids::IdGenerator::new(self.database_node_id)?)?;
    let db = gity_core::Db::connect(&self.database_dsn).await?;
    db.migrate().await?;
    let storage = gity_core::Storage::from_config(&self.storage)?;
    let artifacts = gity_artifacts::ArtifactModule::new(
      fluxdi::Shared::new(db.connection),
      fluxdi::Shared::new(storage.clone()),
    )
    .service()?;
    let authorization = std::sync::Arc::new(
      gity_authority::RemotePackageAuthorization::new(endpoint, timeout)
        .map_err(anyhow::Error::msg)?,
    );
    Ok(Some(gity_package_registry::PackageRegistryModule::new(
      self.address,
      gity_package_registry::PackageState {
        artifacts,
        storage,
        authorization,
      },
    )))
  }
}
