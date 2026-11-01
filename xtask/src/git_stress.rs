use anyhow::{Context, Result, bail};
use clap::{Args, ValueEnum};
use std::path::Path;

mod api;
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

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum Mode {
    Mixed,
    Clone,
    Fetch,
    Push,
    LsRemote,
}

pub struct Config {
    pub base_url: String,
    pub workers: usize,
    pub iterations: usize,
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
    if args.workers == 0 || args.iterations == 0 || args.timeout_ms == 0 {
        bail!("workers, iterations, and timeout-ms must be positive");
    }
    let weights = [
        args.clone_percent,
        args.fetch_percent,
        args.push_percent,
        args.ls_remote_percent,
    ];
    if args.mode == Mode::Mixed && weights.iter().sum::<u32>() == 0 {
        bail!("mixed mode requires at least one positive operation percentage");
    }
    let base_url = if args.base_url.is_empty() {
        format!("http://127.0.0.1:{}", args.port)
    } else {
        args.base_url
    };
    let config = Config {
        base_url,
        workers: args.workers,
        iterations: args.iterations,
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
    workload::run(root, config)
}

pub fn compose_up(root: &Path, project: &str, port: u16, build: bool) -> Result<()> {
    let mut command = std::process::Command::new("docker");
    command.current_dir(root).args([
        "compose",
        "-p",
        project,
        "-f",
        "docker-compose.blackbox.yaml",
        "up",
        "-d",
    ]);
    if build {
        command.arg("--build");
    }
    command.env("GITY_BLACKBOX_WEB_PORT", port.to_string());
    let status = command
        .status()
        .context("start blackbox Docker Compose stack")?;
    if !status.success() {
        bail!("docker compose up failed with {status}");
    }
    Ok(())
}
