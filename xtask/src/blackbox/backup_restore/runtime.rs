use crate::blackbox::process;
use anyhow::{Context, Result, bail};
use std::{
  path::PathBuf,
  process::{Command, Stdio},
};

pub(super) struct Restored {
  pub project: String,
  pub compose: PathBuf,
  assets: PathBuf,
  volumes: Vec<String>,
  cleaned: bool,
}

impl Restored {
  pub fn new(project: String, compose: PathBuf, assets: PathBuf, volumes: Vec<String>) -> Self {
    Self {
      project,
      compose,
      assets,
      volumes,
      cleaned: false,
    }
  }

  /// Successful acceptance requires explicit, verified cleanup. Drop only retries
  /// cleanup after a failure; it is never evidence that cleanup succeeded.
  pub fn finish(mut self) -> Result<()> {
    self.cleanup()?;
    self.cleaned = true;
    Ok(())
  }

  fn cleanup(&self) -> Result<()> {
    let mut first_error = None;
    let mut record = |result: Result<()>| {
      if let Err(error) = result
        && first_error.is_none()
      {
        first_error = Some(error);
      }
    };
    record(
      process::run_output(
        Command::new("docker")
          .args(["compose", "-p", &self.project, "-f"])
          .arg(&self.compose)
          .args(["--profile", "*", "down", "--remove-orphans"]),
        "stop and remove restored project",
      )
      .map(|_| ()),
    );
    for volume in &self.volumes {
      record(
        process::run_output(
          Command::new("docker").args(["volume", "rm", volume]),
          "remove exact restored project volume",
        )
        .map(|_| ()),
      );
    }
    if self.assets.exists() {
      record(clean_assets(&self.assets));
    }
    for kind in ["container", "network", "volume"] {
      record(assert_absent(kind, &self.project));
    }
    if let Some(error) = first_error {
      return Err(error);
    }
    Ok(())
  }
}

impl Drop for Restored {
  fn drop(&mut self) {
    if !self.cleaned {
      let _ = self.cleanup();
    }
  }
}

fn assert_absent(kind: &str, project: &str) -> Result<()> {
  let mut command = Command::new("docker");
  if kind == "container" {
    command.args(["ps", "--all", "--quiet"]);
  } else {
    command.args([kind, "ls", "--quiet"]);
  }
  command.args([
    "--filter",
    &format!("label=com.docker.compose.project={project}"),
  ]);
  let output = process::run_output(&mut command, "verify restored project resource cleanup")?;
  if !output.stdout.iter().all(u8::is_ascii_whitespace) {
    bail!("restored project left {kind} resources after cleanup");
  }
  Ok(())
}

fn clean_assets(assets: &std::path::Path) -> Result<()> {
  let helper = AssetCleanup(format!(
    "gity-backup-cleanup-{}",
    uuid::Uuid::now_v7().simple()
  ));
  process::run_output(
    Command::new("docker")
      .args([
        "run",
        "--rm",
        "--name",
        &helper.0,
        "--label",
        "io.gity.backup.helper=true",
        "--network",
        "none",
        "--mount",
      ])
      .arg(format!("type=bind,src={},dst=/cleanup", assets.display()))
      .args([
        "debian:bookworm-slim",
        "find",
        "/cleanup",
        "-mindepth",
        "1",
        "-delete",
      ]),
    "remove restored private assets",
  )?;
  std::fs::remove_dir_all(assets).context("remove empty restored private directory")?;
  Ok(())
}

struct AssetCleanup(String);
impl Drop for AssetCleanup {
  fn drop(&mut self) {
    let _ = Command::new("docker")
      .args(["rm", "--force", &self.0])
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status();
  }
}

pub(super) fn control(target: &Restored, action: &str) -> Result<()> {
  let mut command = Command::new("docker");
  command
    .args(["compose", "-p", &target.project, "-f"])
    .arg(&target.compose)
    .args(["--profile", "*"]);
  match action {
    "up" => {
      command.args(["up", "--detach", "--no-build"]);
    }
    "restart" => {
      command.arg("restart");
    }
    _ => bail!("unsupported restored project action"),
  }
  process::run_status(&mut command, "control isolated restored project")
}
