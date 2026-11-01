use clap::Parser;
use gity_git_node_app::configuration::{CliOverrides, load_runtime};
use std::{net::SocketAddr, path::PathBuf};

#[derive(Parser)]
#[command(version, about = "Gity dedicated Git storage node")]
struct Cli {
  #[arg(long, env = "GITY_GIT_NODE_CONFIG")]
  config: Option<PathBuf>,
  #[arg(long)]
  id: Option<String>,
  #[arg(long)]
  address: Option<SocketAddr>,
  #[arg(long)]
  repo_root: Option<PathBuf>,
  #[arg(long)]
  git_bin: Option<PathBuf>,
  #[arg(long)]
  hook_executable: Option<PathBuf>,
  #[arg(long, hide = true)]
  token: Option<String>,
  #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
  read_only: Option<bool>,
  #[arg(long)]
  replica_max_bytes: Option<u64>,
  #[arg(long)]
  replica_timeout_secs: Option<u64>,
  #[arg(long)]
  tls_certificate: Option<PathBuf>,
  #[arg(long)]
  tls_key: Option<PathBuf>,
  #[arg(long)]
  client_ca: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  let (settings, observability) = load_runtime(
    cli.config,
    CliOverrides {
      id: cli.id,
      address: cli.address,
      repo_root: cli.repo_root,
      git_bin: cli.git_bin,
      hook_executable: cli.hook_executable,
      token: cli.token,
      read_only: cli.read_only,
      replica_max_bytes: cli.replica_max_bytes,
      replica_timeout_secs: cli.replica_timeout_secs,
      tls_certificate: cli.tls_certificate,
      tls_key: cli.tls_key,
      client_ca: cli.client_ca,
    },
  )?;
  let telemetry = gity_observability::init(&observability)?;
  let mut app = fluxdi::Application::new(gity_git_node::GitNodeModule::new(settings)?);
  app.bootstrap().await?;
  gity_runtime::shutdown_signal().await;
  app.shutdown().await?;
  telemetry.shutdown();
  Ok(())
}
