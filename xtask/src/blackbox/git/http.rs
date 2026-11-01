use crate::blackbox::{Reporter, RunConfig, api, process};
use anyhow::{Context, Result, bail};
use std::{path::Path, process::Command};

pub(super) fn run(
  config: &RunConfig,
  scenario: &api::Scenario,
  work: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let work = work.join("git-http");
  std::fs::create_dir_all(&work).context("create isolated HTTP Git work directory")?;
  let url = format!(
    "{}/{}.git",
    config.http_url.trim_end_matches('/'),
    scenario.project_path
  );
  let first = work.join("owner-clone-one");
  let second = work.join("owner-clone-two");

  run_git(
    &work,
    &scenario.owner_token,
    ["clone", url.as_str(), path(&first)?],
    "HTTP first clone",
  )?;
  run_git(
    &work,
    &scenario.owner_token,
    ["clone", url.as_str(), path(&second)?],
    "HTTP second clone",
  )?;
  configure_author(&first, &scenario.owner_token)?;
  std::fs::write(first.join("http-blackbox.bin"), b"HTTP Git bytes\0\xff\n")?;
  run_git(
    &first,
    &scenario.owner_token,
    ["add", "http-blackbox.bin"],
    "stage HTTP Git fixture",
  )?;
  run_git(
    &first,
    &scenario.owner_token,
    ["commit", "-m", "blackbox HTTP push"],
    "commit HTTP Git fixture",
  )?;
  run_git(
    &first,
    &scenario.owner_token,
    ["push", "origin", "main"],
    "HTTP Git push",
  )?;
  run_git(
    &second,
    &scenario.owner_token,
    ["pull", "--ff-only", "origin", "main"],
    "HTTP Git pull",
  )?;
  let actual = std::fs::read(second.join("http-blackbox.bin"))?;
  if actual != b"HTTP Git bytes\0\xff\n" {
    bail!("second HTTP clone did not receive the exact pushed bytes");
  }

  let outsider = work.join("outsider-clone");
  let result = git_command(&work, &scenario.outsider_token)
    .arg("clone")
    .arg(&url)
    .arg(&outsider)
    .output()
    .context("run outsider HTTP clone")?;
  if result.status.success() {
    bail!("HTTP Git accepted the outsider credential for a private project");
  }
  reporter.pass(
    "http-git-clone-push-pull-outsider-denied",
    "two authenticated HTTP clones exchanged exact file bytes; outsider clone was rejected",
  );
  Ok(())
}

fn configure_author(repository: &Path, token: &str) -> Result<()> {
  run_git(
    repository,
    token,
    ["config", "user.name", "Gity Blackbox"],
    "configure HTTP Git author",
  )?;
  run_git(
    repository,
    token,
    ["config", "user.email", "blackbox@example.test"],
    "configure HTTP Git email",
  )
}

fn run_git<const N: usize>(cwd: &Path, token: &str, args: [&str; N], name: &str) -> Result<()> {
  let mut command = git_command(cwd, token);
  command.args(args);
  process::run_output(&mut command, name).map(|_| ())
}

fn git_command(cwd: &Path, token: &str) -> Command {
  let authorization = format!("Authorization: Bearer {token}");
  let mut command = Command::new("git");
  command
    .current_dir(cwd)
    .env("GIT_CONFIG_NOSYSTEM", "1")
    .env("GIT_CONFIG_GLOBAL", "/dev/null")
    .env("GIT_TERMINAL_PROMPT", "0")
    .env("GIT_CONFIG_COUNT", "2")
    .env("GIT_CONFIG_KEY_0", "http.extraHeader")
    .env("GIT_CONFIG_VALUE_0", authorization)
    .env("GIT_CONFIG_KEY_1", "credential.helper")
    .env("GIT_CONFIG_VALUE_1", "");
  command
}

fn path(value: &Path) -> Result<&str> {
  value.to_str().context("HTTP clone path is not UTF-8")
}
