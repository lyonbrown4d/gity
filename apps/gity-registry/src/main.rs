mod credentials;
mod package_settings;
mod settings;

use clap::{Args, Parser};
use fluxdi::module::Module;
use gity_authority::RemoteOciAuthorization;
use gity_oci::{MirrorResolver, OciAuthorization, OciClientMirrorResolver, OciGatewayModule};
use gity_runtime::{AngosModule, StandaloneModule};
use std::{path::PathBuf, sync::Arc};

#[derive(Parser)]
#[command(version, about = "Gity OCI registry gateway")]
struct Cli {
  #[arg(long, env = "GITY_CONFIG")]
  config: Option<PathBuf>,
  #[arg(
    long,
    env = "GITY_AUTHORITY_PEER_TOKEN",
    hide_env_values = true,
    default_value = ""
  )]
  authority_peer_token: String,
  #[command(flatten)]
  overrides: RegistryCliOverrides,
}

#[derive(Args, Default)]
struct RegistryCliOverrides {
  #[arg(long)]
  address: Option<std::net::SocketAddr>,
  #[arg(long)]
  request_timeout_seconds: Option<u64>,
  #[arg(long)]
  angos_executable: Option<PathBuf>,
  #[arg(long)]
  authority_uri: Option<String>,
  #[arg(long)]
  oci_upstream: Option<String>,
}

type ModuleFactory = Arc<dyn Fn() -> Box<dyn Module> + Send + Sync>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  let overrides = settings::RegistryOverrides {
    address: cli.overrides.address,
    request_timeout_seconds: cli.overrides.request_timeout_seconds,
    angos_executable: cli.overrides.angos_executable,
    authority_uri: cli.overrides.authority_uri,
    oci_upstream: cli.overrides.oci_upstream,
  };
  let settings = settings::RegistrySettings::load(cli.config.as_deref(), overrides)?
    .into_runtime(cli.authority_peer_token)
    .await?;
  let telemetry = gity_observability::init(&settings.observability)?;
  let package_module = settings
    .packages
    .module(
      settings.authority_endpoint.clone(),
      settings.oci.request_timeout,
    )
    .await?;
  let packages_enabled = package_module.is_some();
  let authorization: Arc<dyn OciAuthorization> = Arc::new(
    RemoteOciAuthorization::new(settings.authority_endpoint, settings.oci.request_timeout)
      .map_err(anyhow::Error::msg)?,
  );
  let mirror_resolver: Option<Arc<dyn MirrorResolver>> = if settings.oci.mirror.is_some() {
    Some(Arc::new(
      OciClientMirrorResolver::new(&settings.oci, settings.credentials)
        .map_err(anyhow::Error::msg)?,
    ))
  } else {
    None
  };
  let gateway_address = settings.address;
  let oci_settings = settings.oci;
  let angos_settings = settings.angos;
  let gateway_authorization = authorization.clone();
  let gateway_resolver = mirror_resolver.clone();
  let mut components: Vec<ModuleFactory> = vec![
    Arc::new(move || {
      Box::new(
        AngosModule::new(angos_settings.clone())
          .expect("Angos configuration was validated before module composition"),
      )
    }),
    Arc::new(move || {
      Box::new(
        OciGatewayModule::new_with_mirror(
          gateway_address,
          oci_settings.clone(),
          gateway_authorization.clone(),
          gateway_resolver.clone(),
        )
        .expect("OCI configuration was validated before module composition"),
      )
    }),
  ];
  if let Some(module) = package_module {
    components.push(Arc::new(move || Box::new(module.clone())));
  }
  let mut application = fluxdi::Application::new(StandaloneModule::new(components));
  application.bootstrap().await?;
  tracing::info!(address = %gateway_address, "Gity OCI registry is ready");
  let failed = tokio::select! {
    _ = gity_runtime::shutdown_signal() => false,
    _ = monitor(application.injector(),packages_enabled) => true,
  };
  application.shutdown().await?;
  telemetry.shutdown();
  anyhow::ensure!(!failed, "registry component stopped unexpectedly");
  Ok(())
}

async fn monitor(injector: fluxdi::Shared<fluxdi::Injector>, packages_enabled: bool) {
  let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
  loop {
    interval.tick().await;
    if packages_enabled
      && !injector
        .try_resolve::<gity_package_registry::PackageRegistryHealth>()
        .is_ok_and(|health| health.is_healthy())
    {
      return;
    }

    if !injector
      .try_resolve::<gity_runtime::AngosProcess>()
      .is_ok_and(|process| !process.has_failed())
    {
      return;
    }
    if !injector
      .try_resolve::<gity_oci::OciGatewayHealth>()
      .is_ok_and(|health| health.is_healthy())
    {
      return;
    }
  }
}
