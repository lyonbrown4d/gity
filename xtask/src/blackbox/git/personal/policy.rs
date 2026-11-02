use super::{
  api::{self, Account},
  session::Master,
};
use crate::blackbox::{RunConfig, api as http};
use anyhow::Result;
use reqwest::{Method, blocking::Client};
use serde_json::json;
use std::path::Path;

pub(super) fn run(
  client: &Client,
  config: &RunConfig,
  scenario: &http::Scenario,
  account: &Account,
  master: &Master,
  repository: &Path,
) -> Result<()> {
  // The preceding deployment suite protects main for maintainers. The same
  // authenticated connection must gain and lose effective permissions live.
  master.git(
    repository,
    ["push", "origin", "HEAD:refs/heads/main"],
    false,
    "developer protected main push denied",
  )?;
  api::role(client, config, scenario, account, Some("maintainer"))?;
  master.git(
    repository,
    ["push", "origin", "HEAD:refs/heads/main"],
    true,
    "promoted maintainer protected main push",
  )?;
  master.git(
    repository,
    ["push", "--force", "origin", "HEAD~1:refs/heads/main"],
    false,
    "personal protected force push denied",
  )?;
  master.git(
    repository,
    ["push", "origin", ":refs/heads/main"],
    false,
    "personal default branch deletion denied",
  )?;
  protect(client, config, scenario, true)?;
  std::fs::write(
    repository.join("personal-ssh.txt"),
    "personal SSH merge request boundary\n",
  )?;
  let mut add = std::process::Command::new("git");
  add
    .current_dir(repository)
    .args(["add", "personal-ssh.txt"]);
  crate::blackbox::process::run_status(&mut add, "stage personal merge-request boundary")?;
  let mut commit = std::process::Command::new("git");
  commit
    .current_dir(repository)
    .args(["commit", "-m", "personal SSH merge request boundary"]);
  crate::blackbox::process::run_status(&mut commit, "commit personal merge-request boundary")?;
  master.git(
    repository,
    ["push", "origin", "HEAD:refs/heads/main"],
    false,
    "personal require-merge-request push denied",
  )?;
  protect(client, config, scenario, false)?;
  api::role(client, config, scenario, account, Some("reporter"))?;
  master.git(
    repository,
    ["push", "origin", "HEAD:refs/heads/personal-after-downgrade"],
    false,
    "downgraded reporter unprotected push denied",
  )?;
  master.git(
    repository,
    ["ls-remote", "origin"],
    true,
    "reporter retains personal repository read",
  )?;
  api::role(client, config, scenario, account, None)?;
  master.git(
    repository,
    ["ls-remote", "origin"],
    false,
    "removed membership blocks existing SSH connection",
  )?;
  api::role(client, config, scenario, account, Some("developer"))?;
  master.git(
    repository,
    ["ls-remote", "origin"],
    true,
    "restored membership immediately permits repository read",
  )?;
  Ok(())
}
fn protect(
  client: &Client,
  config: &RunConfig,
  scenario: &http::Scenario,
  require_merge_request: bool,
) -> Result<()> {
  api::ok(
    client,
    config,
    Method::POST,
    &format!(
      "/projects/{}/repository/branches/main/protect",
      scenario.project_id
    ),
    Some(&scenario.owner_token),
    Some(json!({
      "push_access_level":"maintainer", "merge_access_level":"maintainer",
      "require_merge_request":require_merge_request, "require_pipeline_success":false,
      "allow_force_push":false, "allow_delete":false
    })),
    "set personal SSH protected branch policy",
  )?;
  Ok(())
}
