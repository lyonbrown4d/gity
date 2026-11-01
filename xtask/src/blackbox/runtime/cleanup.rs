use super::super::{BlackboxArgs, ComposeAction, Reporter, process};
use super::compose;
use anyhow::{Context, Result, bail};
use std::{path::Path, process::Command};
use tempfile::TempDir;

pub fn ensure_unclaimed_project(args: &BlackboxArgs) -> Result<()> {
  let project_label = format!("label=com.docker.compose.project={}", args.project);
  let mut containers = Command::new("docker");
  containers.args(["ps", "--all", "--quiet", "--filter", &project_label]);
  assert_empty(args, "containers", &mut containers)?;
  let mut networks = Command::new("docker");
  networks.args(["network", "ls", "--quiet", "--filter", &project_label]);
  assert_empty(args, "networks", &mut networks)?;
  let mut volumes = Command::new("docker");
  volumes.args(["volume", "ls", "--quiet", "--filter", &project_label]);
  assert_empty(args, "volumes", &mut volumes)?;
  Ok(())
}

fn assert_empty(args: &BlackboxArgs, kind: &str, command: &mut Command) -> Result<()> {
  let output = process::run_output(command, "check black-box project ownership")?;
  if !output.stdout.is_empty() {
    bail!(
      "black-box project {} already has {kind}; refusing to inspect or clean it",
      args.project
    );
  }
  Ok(())
}

pub fn finalize(
  root: &Path,
  args: &BlackboxArgs,
  work: TempDir,
  compose_attempted: bool,
  evidence_path: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  if args.compose_action == ComposeAction::Up && compose_attempted {
    preserve_workdir(work, reporter);
    reporter.write(evidence_path)?;
    return Ok(());
  }

  if compose_attempted && let Err(error) = compose(root, args, "down") {
    reporter.failure(&error);
    preserve_workdir(work, reporter);
    reporter.write(evidence_path)?;
    return Err(error).context("stop owned black-box Compose project");
  }
  if let Err(error) = clear_workdir(work.path()) {
    reporter.failure(&error);
    preserve_workdir(work, reporter);
    reporter.write(evidence_path)?;
    return Err(error).context("clean private black-box work directory");
  }
  reporter.write(evidence_path)
}

fn preserve_workdir(work: TempDir, reporter: &Reporter) {
  let path = work.keep();
  reporter.note(&format!(
    "Preserved private black-box work directory: {}",
    path.display()
  ));
}

fn clear_workdir(path: &Path) -> Result<()> {
  let mut command = Command::new("docker");
  command
    .args(["run", "--rm", "--user", "0:0", "--volume"])
    .arg(format!("{}:/work", path.display()))
    .args([
      "alpine:3.22",
      "sh",
      "-c",
      "rm -rf /work/* /work/.[!.]* /work/..?*",
    ]);
  process::run_status(&mut command, "remove task-owned black-box work files")
    .context("run isolated cleanup helper")?;
  Ok(())
}
