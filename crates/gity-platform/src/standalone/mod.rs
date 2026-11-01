#[cfg(feature = "oci")]
mod engine;
mod health;
pub mod runner;
mod secrets;
use crate::{
  AppState,
  lifecycle::{PlatformModule, PlatformOptions, shutdown_signal},
};
use anyhow::Result;
use fluxdi::module::Module;
#[cfg(feature = "oci")]
use gity_oci::{
  MirrorRegistry, MirrorSettings, OciClientMirrorResolver, OciGatewayModule, OciSettings,
  UpstreamPolicy,
};
use std::sync::Arc;
#[cfg(feature = "oci")]
use std::time::Duration;

pub async fn run(state: AppState, mut options: PlatformOptions, full: bool) -> Result<()> {
  if options.migrate {
    state.db.migrate().await?;
    options.migrate = false;
  }
  let platform = PlatformModule::new(state.clone(), options);
  let mut failures = platform.subscribe();
  let mut factories: Vec<Arc<dyn Fn() -> Box<dyn Module> + Send + Sync>> = Vec::new();
  #[cfg(feature = "oci")]
  if full && state.settings.components.registry.enabled {
    let engine =
      gity_runtime::AngosModule::new(engine::config(&state).await?).map_err(anyhow::Error::msg)?;
    factories.push(Arc::new(move || Box::new(engine.clone())));
  }
  factories.push(Arc::new(move || Box::new(platform.clone())));
  #[cfg(feature = "oci")]
  if state.settings.components.authority.enabled {
    let config = &state.settings.components.authority;
    let module = gity_authority::AuthorityModule::new(
      gity_authority::AuthoritySettings {
        address: config.address,
        peer_token: config.token.clone(),
        tls_certificate: config.tls_certificate.clone(),
        tls_key: config.tls_key.clone(),
        client_ca: config.client_ca.clone(),
      },
      Arc::new(crate::oci_registry::PlatformOciAuthorizer::new(
        state.clone(),
      )),
    )
    .map_err(anyhow::Error::msg)?;
    #[cfg(feature = "packages")]
    let module = module.with_packages(Arc::new(
      crate::package_authority::PlatformPackageAuthority::new(state.clone()),
    ));
    factories.push(Arc::new(move || Box::new(module.clone())));
  }
  #[cfg(feature = "oci")]
  if full && state.settings.components.registry.enabled {
    let config = &state.settings.components.registry;
    let mirror = (!config.mirrors.is_empty()).then(|| MirrorSettings {
      registries: config
        .mirrors
        .iter()
        .map(|(alias, value)| {
          (
            alias.clone(),
            MirrorRegistry {
              host: value.host.clone(),
              credential_secret_ref: value.credential_secret_ref.clone(),
            },
          )
        })
        .collect(),
      allowed_hosts: config.allowed_upstream_authorities.clone(),
    });
    let settings = OciSettings {
      upstream: config.engine_url.clone(),
      request_timeout: Duration::from_secs(300),
      upstream_policy: UpstreamPolicy::default(),
      mirror,
    };
    let resolver = if settings.mirror.is_some() {
      Some(Arc::new(
        OciClientMirrorResolver::new(&settings, Arc::new(secrets::ConfiguredSecrets))
          .map_err(anyhow::Error::msg)?,
      ) as Arc<dyn gity_oci::MirrorResolver>)
    } else {
      None
    };
    let module = OciGatewayModule::new_with_mirror(
      config.address,
      settings,
      Arc::new(crate::oci_registry::PlatformOciAuthorizer::new(
        state.clone(),
      )),
      resolver,
    )
    .map_err(anyhow::Error::msg)?;
    factories.push(Arc::new(move || Box::new(module.clone())));
  }
  if full && state.settings.components.runner.enabled {
    let module = runner::RunnerModule::new(state.clone());
    factories.push(Arc::new(move || Box::new(module.clone())));
  }
  let mut app = fluxdi::Application::new(gity_runtime::StandaloneModule::new(factories));
  app.bootstrap().await?;
  let failure = tokio::select! {
    _=shutdown_signal()=>None,
    result=failures.changed()=>{result?;failures.borrow().clone()},
    error=health::monitor(app.injector(), &state.settings, full)=>Some(error),
  };
  let stopped = app.shutdown().await;
  stopped?;
  if let Some(error) = failure {
    anyhow::bail!("application component failed: {error}");
  }
  Ok(())
}
