use super::{GitCredential, commands};
use crate::blackbox::{Reporter, RunConfig, api};
use anyhow::{Context, Result, bail};
use std::path::Path;

pub fn after_promotion(
  config: &RunConfig,
  scenario: &api::Scenario,
  credential: &GitCredential,
  work: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let clone = work.join("git-after-promotion");
  let url = format!(
    "ssh://git@{}:{}/{}.git",
    config.ssh_host, config.ssh_port, scenario.project_path
  );
  commands::run_git(
    work,
    &credential.private_key,
    &credential.known_hosts,
    [
      "clone",
      url.as_str(),
      clone.to_str().context("clone path is not UTF-8")?,
    ],
    "SSH clone after controlled promotion",
  )?;
  if std::fs::read_to_string(clone.join("ssh-blackbox.txt"))? != "pushed by the first clone\n" {
    bail!("promoted Git read did not return the expected file");
  }
  commands::configure(&clone, &credential.private_key, &credential.known_hosts)?;
  std::fs::write(clone.join("after-promotion.txt"), "write after promotion\n")?;
  commands::run_git(
    &clone,
    &credential.private_key,
    &credential.known_hosts,
    ["add", "."],
    "stage after-promotion write",
  )?;
  commands::run_git(
    &clone,
    &credential.private_key,
    &credential.known_hosts,
    ["commit", "-m", "blackbox after promotion"],
    "commit after-promotion write",
  )?;
  commands::run_git(
    &clone,
    &credential.private_key,
    &credential.known_hosts,
    ["push", "origin", "HEAD:refs/heads/blackbox-after-promotion"],
    "push after controlled promotion",
  )?;
  reporter.pass(
    "distributed-git-read-write-after-promotion",
    "SSH read and branch write succeeded after promotion",
  );
  Ok(())
}
