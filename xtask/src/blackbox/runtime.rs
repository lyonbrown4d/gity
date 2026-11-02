use super::{Base, BlackboxArgs, Mode, Reporter, RunConfig, process};
use anyhow::{Context, Result, bail};
use std::{
  path::{Path, PathBuf},
  process::Command,
  time::{Duration, Instant},
};

mod cleanup;
mod deployment;
mod evidence;
pub(super) use cleanup::{ensure_unclaimed_project, finalize};
pub(super) use deployment::{prepare_environment, remember_environment_secrets};
pub(super) use evidence::{capture_failure_logs, capture_images};
pub fn grant_admin(root: &Path, args: &BlackboxArgs, username: &str) -> Result<()> {
  let mut command = Command::new("docker");
  configure_compose(&mut command, root, args)?;
  command.args([
    "exec",
    "--no-TTY",
    if args.mode == Mode::Standalone {
      "gity"
    } else {
      "api"
    },
    "/usr/local/bin/gity",
    "--config",
    "/etc/gity/gity.toml",
    "grant-admin",
    username,
  ]);
  process::run_status(&mut command, "black-box admin grant")
}

pub(super) fn compose_file(root: &Path, mode: Mode) -> PathBuf {
  root.join("deploy/components").join(match mode {
    Mode::Standalone => "standalone.compose.yaml",
    Mode::Distributed => "distributed.compose.yaml",
  })
}

pub(super) fn configure_compose(
  command: &mut Command,
  root: &Path,
  args: &BlackboxArgs,
) -> Result<()> {
  command.current_dir(root).args([
    "compose",
    "--project-name",
    args.project.as_str(),
    "--file",
    compose_file(root, args.mode)
      .to_str()
      .context("Compose file path is not UTF-8")?,
  ]);
  if let Some(env_file) = &args.env_file {
    command.arg("--env-file").arg(env_file);
  }
  command.env("GITY_BASE", base_name(args.base));
  command.env(
    "GITY_BUILDER_IMAGE",
    match args.base {
      Base::Debian => "rust:1.99.1-bookworm",
      Base::Alpine => "rust:1.99.1-alpine",
    },
  );
  command.env(
    "GITY_RUNTIME_IMAGE",
    match args.base {
      Base::Debian => "debian:bookworm-slim",
      Base::Alpine => "alpine:3.22",
    },
  );
  set_image_names(command, &args.project, args.mode, args.base);
  Ok(())
}

pub(super) fn compose(root: &Path, args: &BlackboxArgs, action: &str) -> Result<()> {
  let mut command = Command::new("docker");
  configure_compose(&mut command, root, args)?;
  match action {
    "up" => {
      command.args([
        "up",
        "--detach",
        if args.no_build {
          "--no-build"
        } else {
          "--build"
        },
      ]);
    }
    "down" => {
      command.args(["down", "--volumes", "--remove-orphans"]);
    }
    _ => bail!("unsupported Compose action"),
  }
  process::run_status(&mut command, "docker compose")
}

pub(super) fn start_distributed_runner(
  root: &Path,
  args: &BlackboxArgs,
  runner_token: &str,
) -> Result<()> {
  if args.mode != Mode::Distributed {
    bail!("distributed Runner can only start in distributed mode");
  }
  let mut command = Command::new("docker");
  configure_compose(&mut command, root, args)?;
  command.env("GITY_DISTRIBUTED_RUNNER_TOKEN", runner_token);
  command.args([
    "--profile",
    "runner",
    "up",
    "--detach",
    if args.no_build {
      "--no-build"
    } else {
      "--build"
    },
    "docker",
    "runner",
  ]);
  process::run_status(&mut command, "start distributed black-box Runner image")
}

fn set_image_names(command: &mut Command, project: &str, mode: Mode, base: Base) {
  let suffix = base_name(base);
  let names = match mode {
    Mode::Standalone => vec![("GITY_STANDALONE_IMAGE", "standalone")],
    Mode::Distributed => vec![
      ("GITY_API_IMAGE", "api"),
      ("GITY_GIT_IMAGE", "git"),
      ("GITY_GIT_NODE_IMAGE", "git-node"),
      ("GITY_GATEWAY_IMAGE", "gateway"),
      ("GITY_REGISTRY_IMAGE", "registry"),
      ("GITY_WORKER_IMAGE", "worker"),
      ("GITY_RUNNER_IMAGE", "runner"),
    ],
  };
  for (key, role) in names {
    command.env(key, format!("gity-blackbox:{project}-{role}-{suffix}"));
  }
}

fn base_name(base: Base) -> &'static str {
  match base {
    Base::Debian => "debian",
    Base::Alpine => "alpine",
  }
}

pub(super) fn wait_for_health(config: &RunConfig, reporter: &mut Reporter) -> Result<()> {
  let client = reqwest::blocking::Client::builder()
    .timeout(Duration::from_secs(10))
    .build()?;
  let deadline = Instant::now() + Duration::from_secs(180);
  while Instant::now() < deadline {
    if client
      .get(format!("{}/api/health", config.http_url))
      .send()
      .is_ok_and(|response| response.status().is_success())
    {
      reporter.pass("http-health", "healthy");
      return Ok(());
    }
    std::thread::sleep(Duration::from_secs(2));
  }
  bail!("HTTP health endpoint did not become ready")
}

pub(super) fn control_service(
  root: &Path,
  args: &BlackboxArgs,
  action: &str,
  service: &str,
) -> Result<()> {
  if !matches!(action, "start" | "stop") {
    bail!("unsupported Compose service action");
  }
  let mut command = Command::new("docker");
  configure_compose(&mut command, root, args)?;
  command.args([action, service]);
  process::run_status(&mut command, "control Compose service")
}
