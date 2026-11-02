use crate::blackbox::{RunConfig, api, git};
use anyhow::{Context, Result, bail};
use std::path::Path;

/// Update an existing CI file through real SSH, retaining the original pinned
/// known_hosts and deploy key. The create-only file API is not an update path.
pub(super) fn push(
  config: &RunConfig,
  scenario: &api::Scenario,
  work: &Path,
  branch: &str,
  yaml: &str,
) -> Result<String> {
  let credential = scenario
    .git_credential
    .as_ref()
    .context("restored CI needs pinned SSH credentials")?;
  let repository = work.join("git-restored-ci");
  let url = format!(
    "ssh://git@{}:{}/{}.git",
    config.ssh_host, config.ssh_port, scenario.project_path
  );
  git::commands::run_git(
    work,
    &credential.private_key,
    &credential.known_hosts,
    [
      "clone",
      "--single-branch",
      "--branch",
      branch,
      &url,
      repository
        .to_str()
        .context("restored CI clone path must be UTF-8")?,
    ],
    "strict-pinned SSH clone for restored CI",
  )?;
  git::commands::configure(
    &repository,
    &credential.private_key,
    &credential.known_hosts,
  )?;
  let previous = head(&repository, credential)?;
  let file = repository.join(".gity-ci.yml");
  if !file.is_file() {
    bail!("restored repository lost the original CI configuration");
  }
  std::fs::write(file, yaml).context("update restored CI YAML in SSH clone")?;
  git::commands::run_git(
    &repository,
    &credential.private_key,
    &credential.known_hosts,
    ["add", ".gity-ci.yml"],
    "stage new restored CI configuration",
  )?;
  git::commands::run_git(
    &repository,
    &credential.private_key,
    &credential.known_hosts,
    ["commit", "-m", "Run new CI work after offline restore"],
    "commit new restored CI configuration",
  )?;
  let sha = head(&repository, credential)?;
  if sha == previous {
    bail!("restored CI must execute a new commit");
  }
  let target = format!("HEAD:refs/heads/{branch}");
  git::commands::run_git(
    &repository,
    &credential.private_key,
    &credential.known_hosts,
    ["push", "origin", &target],
    "strict-pinned SSH push after offline restore",
  )?;
  Ok(sha)
}

fn head(repository: &Path, credential: &git::GitCredential) -> Result<String> {
  let output = git::commands::output(
    repository,
    &credential.private_key,
    &credential.known_hosts,
    ["rev-parse", "HEAD"],
  )?;
  let sha = String::from_utf8(output)
    .context("restored CI commit SHA must be UTF-8")?
    .trim()
    .to_owned();
  if sha.len() != 40 || !sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
    bail!("restored CI commit SHA is invalid");
  }
  Ok(sha)
}
