mod settings;

use clap::{ArgAction, Parser};
use gity_gateway::{GatewayModule, GatewaySettings};
use settings::GatewayOverrides;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Parser)]
#[command(version, about = "Gity HTTP and SSH edge gateway")]
struct Cli {
  #[arg(long, env = "GITY_CONFIG")]
  config: Option<PathBuf>,
  #[arg(long)]
  http_listen: Option<SocketAddr>,
  #[arg(long)]
  ssh_listen: Option<SocketAddr>,
  #[arg(long)]
  main_http_origin: Option<String>,
  #[arg(long)]
  package_http_origin: Option<String>,
  #[arg(long,value_parser=["http","https"])]
  public_scheme: Option<String>,
  #[arg(long)]
  main_ssh_address: Option<String>,
  #[arg(long, action = ArgAction::Set)]
  allow_insecure_private_network: Option<bool>,
  #[arg(long)]
  request_timeout_seconds: Option<u64>,
  #[arg(long)]
  ssh_connect_timeout_seconds: Option<u64>,
  #[arg(long)]
  ssh_session_timeout_seconds: Option<u64>,
  #[arg(long)]
  shutdown_timeout_seconds: Option<u64>,
  #[arg(long)]
  max_http_requests: Option<usize>,
  #[arg(long)]
  max_ssh_sessions: Option<usize>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  let overrides = GatewayOverrides {
    http_listen: cli.http_listen,
    ssh_listen: cli.ssh_listen,
    main_http_origin: cli.main_http_origin,
    package_http_origin: cli.package_http_origin,
    public_scheme: cli.public_scheme,
    main_ssh_address: cli.main_ssh_address,
    allow_insecure_private_network: cli.allow_insecure_private_network,
    request_timeout_seconds: cli.request_timeout_seconds,
    ssh_connect_timeout_seconds: cli.ssh_connect_timeout_seconds,
    ssh_session_timeout_seconds: cli.ssh_session_timeout_seconds,
    shutdown_timeout_seconds: cli.shutdown_timeout_seconds,
    max_http_requests: cli.max_http_requests,
    max_ssh_sessions: cli.max_ssh_sessions,
  };
  let config = settings::GatewayFile::load(cli.config.as_deref(), overrides)?;
  let telemetry = gity_observability::init(&config.observability)?;
  let settings: GatewaySettings = config.into_runtime()?;
  let module = GatewayModule::new(settings.clone()).map_err(anyhow::Error::msg)?;
  let mut application = fluxdi::Application::new(module);
  application.bootstrap().await?;
  tracing::info!(http = %settings.http_listen, ssh = %settings.ssh_listen, "Gity gateway is ready");
  gity_runtime::shutdown_signal().await;
  application.shutdown().await?;
  telemetry.shutdown();
  Ok(())
}
