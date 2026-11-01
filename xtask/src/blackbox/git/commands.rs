use super::super::process;
use anyhow::{Context, Result};
use std::{path::Path, process::Command};

pub(super) fn configure(repo: &Path, key: &Path, known_hosts: &Path) -> Result<()> {
  run_git(
    repo,
    key,
    known_hosts,
    ["config", "user.name", "Gity Blackbox"],
    "configure Git author",
  )?;
  run_git(
    repo,
    key,
    known_hosts,
    ["config", "user.email", "blackbox@example.test"],
    "configure Git email",
  )
}

pub(super) fn run_git<const N: usize>(
  cwd: &Path,
  key: &Path,
  known_hosts: &Path,
  args: [&str; N],
  name: &str,
) -> Result<()> {
  let mut command = git_command(cwd, key, known_hosts);
  command.args(args);
  process::run_status(&mut command, name)
}

pub(super) fn git_status<const N: usize>(
  cwd: &Path,
  key: &Path,
  known_hosts: &Path,
  args: [&str; N],
) -> Result<bool> {
  let mut command = git_command(cwd, key, known_hosts);
  command.args(args);
  Ok(
    command
      .output()
      .context("run Git negative case")?
      .status
      .success(),
  )
}

fn git_command(cwd: &Path, key: &Path, known_hosts: &Path) -> Command {
  let ssh = format!(
    "ssh -F /dev/null -i {} -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile={} -o ConnectTimeout=8",
    key.display(),
    known_hosts.display()
  );
  let mut command = Command::new("git");
  command
    .current_dir(cwd)
    .env("GIT_SSH_COMMAND", ssh)
    .env("GIT_TERMINAL_PROMPT", "0")
    .env("GIT_CONFIG_NOSYSTEM", "1");
  command
}
