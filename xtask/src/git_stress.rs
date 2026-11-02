use anyhow::{Context, Result, bail};
use clap::{Args, ValueEnum};
use std::path::{Path, PathBuf};

mod api;
mod seed;
mod summary;
mod transport;
mod workload;

#[derive(Clone, Args)]
pub struct GitStressArgs {
  #[arg(long)]
  build: bool,
  #[arg(long)]
  no_compose: bool,
  #[arg(long, default_value = "")]
  base_url: String,
  #[arg(long, default_value_t = 18080)]
  port: u16,
  #[arg(long, default_value = "gity-blackbox")]
  project: String,
  #[arg(long, default_value_t = 4)]
  workers: usize,
  #[arg(long, default_value_t = 24)]
  iterations: usize,
  #[arg(long)]
  duration_seconds: Option<u64>,
  #[arg(long)]
  summary: Option<PathBuf>,
  #[arg(long, value_enum, default_value_t = Transport::Http)]
  transport: Transport,
  #[arg(long, required_if_eq("transport", "ssh"))]
  ssh_key: Option<PathBuf>,
  #[arg(long, required_if_eq("transport", "ssh"))]
  ssh_known_hosts: Option<PathBuf>,
  #[arg(long, default_value_t = 2222)]
  ssh_port: u16,
  #[arg(long)]
  ssh_host: Option<String>,
  #[arg(long, default_value_t = 1)]
  clone_depth: usize,
  #[arg(long)]
  bytes_per_file: Option<u64>,
  #[arg(long, default_value_t = 1)]
  seed_commits: usize,
  #[arg(long, value_enum, default_value_t = Mode::Mixed)]
  mode: Mode,
  #[arg(long, default_value_t = 20)]
  clone_percent: u32,
  #[arg(long, default_value_t = 45)]
  fetch_percent: u32,
  #[arg(long, default_value_t = 25)]
  push_percent: u32,
  #[arg(long, default_value_t = 10)]
  ls_remote_percent: u32,
  #[arg(long, default_value_t = 60000)]
  timeout_ms: u64,
  #[arg(long, default_value = ".tmp-perf/git-client")]
  workdir: String,
  #[arg(long)]
  keep_workdir: bool,
  #[arg(long, default_value_t = 8)]
  seed_files: usize,
  #[arg(long, default_value = "")]
  project_url: String,
  #[arg(long, default_value = "GITY_GIT_PERF_TOKEN")]
  token_env: String,
  #[arg(long, default_value = "git-perf")]
  token_username: String,
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
  Mixed,
  Clone,
  Fetch,
  Push,
  LsRemote,
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
  Http,
  Ssh,
}

pub struct Config {
  pub base_url: String,
  pub workers: usize,
  pub iterations: usize,
  pub duration_seconds: Option<u64>,
  pub summary: Option<PathBuf>,
  pub transport: Transport,
  pub ssh_key: Option<PathBuf>,
  pub ssh_known_hosts: Option<PathBuf>,
  pub ssh_port: u16,
  pub ssh_host: Option<String>,
  pub clone_depth: usize,
  pub bytes_per_file: Option<u64>,
  pub seed_commits: usize,
  pub mode: Mode,
  pub weights: [u32; 4],
  pub timeout_ms: u64,
  pub workdir: String,
  pub keep_workdir: bool,
  pub seed_files: usize,
  pub project_url: String,
  pub token_env: String,
  pub token_username: String,
  pub compose: bool,
  pub build: bool,
  pub project: String,
}

pub fn run(root: &Path, args: GitStressArgs) -> Result<()> {
  if args.workers == 0
    || (args.duration_seconds.is_none() && args.iterations == 0)
    || args.timeout_ms == 0
    || args.duration_seconds == Some(0)
    || args.bytes_per_file == Some(0)
    || args.seed_commits == 0
    || args.ssh_port == 0
  {
    bail!("workers, iteration/window bounds, timeout, seed sizes and ports must be positive");
  }
  let weights = [
    args.clone_percent,
    args.fetch_percent,
    args.push_percent,
    args.ls_remote_percent,
  ];
  if args.mode == Mode::Mixed && weights.iter().all(|&weight| weight == 0) {
    bail!("mixed mode requires at least one positive operation percentage");
  }
  let base_url = if args.base_url.is_empty() {
    format!("http://127.0.0.1:{}", args.port)
  } else {
    args.base_url
  };
  let mut config = Config {
    base_url,
    workers: args.workers,
    iterations: args.iterations,
    duration_seconds: args.duration_seconds,
    summary: args.summary,
    transport: args.transport,
    ssh_key: args.ssh_key,
    ssh_known_hosts: args.ssh_known_hosts,
    ssh_port: args.ssh_port,
    ssh_host: args.ssh_host,
    clone_depth: args.clone_depth,
    bytes_per_file: args.bytes_per_file,
    seed_commits: args.seed_commits,
    mode: args.mode,
    weights,
    timeout_ms: args.timeout_ms,
    workdir: args.workdir,
    keep_workdir: args.keep_workdir,
    seed_files: args.seed_files,
    project_url: args.project_url,
    token_env: args.token_env,
    token_username: args.token_username,
    compose: !args.no_compose,
    build: args.build,
    project: args.project,
  };
  transport::validate(&mut config)?;
  workload::run(root, config)
}

pub fn compose_up(root: &Path, config: &Config, port: u16) -> Result<()> {
  let mut command = std::process::Command::new("docker");
  command.current_dir(root).args([
    "compose",
    "-p",
    &config.project,
    "-f",
    "docker-compose.blackbox.yaml",
  ]);
  let overlay = if config.transport == Transport::Ssh {
    use std::io::Write;
    let mut file = tempfile::NamedTempFile::new()?;
    write!(
      file,
      "services:\n  gity:\n    environment:\n      GITY_SSH__ENABLED: 'true'\n      GITY_SSH__ADDRESS: 0.0.0.0:2222\n      GITY_SSH__HOST_KEY: /var/lib/gity/ssh_host_ed25519\n    ports:\n      - '127.0.0.1:{}:2222'\n",
      config.ssh_port
    )?;
    command.arg("-f").arg(file.path());
    Some(file)
  } else {
    None
  };
  command.args(["up", "-d"]);
  if config.build {
    command.arg("--build");
  }
  command.env("GITY_BLACKBOX_WEB_PORT", port.to_string());
  let status = command
    .status()
    .context("start blackbox Docker Compose stack")?;
  drop(overlay);
  if !status.success() {
    bail!("docker compose up failed with {status}");
  }
  Ok(())
}
