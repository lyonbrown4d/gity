//! Real-image personal account SSH acceptance. The original deploy credential
//! remains registered and unchanged for backup/restore and promotion suites.
mod api;
mod policy;
mod session;
use super::{GitCredential, commands, create_key};
use crate::blackbox::{Reporter, RunConfig, api as http};
use anyhow::{Context, Result, bail};
use reqwest::{Method, StatusCode, blocking::Client};
use serde_json::json;
use std::path::Path;

pub(in crate::blackbox) fn run(
  config: &RunConfig,
  scenario: &http::Scenario,
  credential: &GitCredential,
  work: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let client = Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
  let mut account = api::register(&client, config, &scenario.suffix, reporter)?;
  api::ok(
    &client,
    config,
    Method::POST,
    &format!("/projects/{}/members", scenario.project_id),
    Some(&scenario.owner_token),
    Some(json!({"user_id":account.id,"role":"developer"})),
    "owner adds ordinary developer",
  )?;
  let personal_work = work.join("personal-ssh");
  std::fs::create_dir_all(&personal_work)?;
  let key = personal_work.join("key-ed25519");
  let public = create_key(&key)?;
  let registered = api::ok(
    &client,
    config,
    Method::POST,
    "/users/me/ssh-keys",
    Some(&account.token),
    Some(json!({"title":"personal blackbox","public_key":format!("{public} workstation")})),
    "register personal public SSH key",
  )?;
  let key_id = http::id(&registered["id"], "registered personal SSH key")?;
  if registered["public_key"] != public
    || registered["user_id"] != account.id
    || !registered["fingerprint"]
      .as_str()
      .is_some_and(|v| v.starts_with("SHA256:"))
    || registered["created_at"].as_str().is_none()
    || !registered["last_used_at"].is_null()
    || registered.get("private_key").is_some()
  {
    bail!("personal SSH key metadata was not canonical public material");
  }
  let key_path = format!("/users/me/ssh-keys/{key_id}");
  api::expect(
    &client,
    config,
    Method::DELETE,
    &key_path,
    &scenario.owner_token,
    json!(null),
    StatusCode::NOT_FOUND,
    "owner/admin cannot delete another account's SSH key",
  )?;
  api::negative_inputs(
    &client,
    config,
    scenario,
    &account,
    &key,
    &public,
    &credential.private_key,
    reporter,
  )?;
  let url = format!(
    "ssh://git@{}:{}/{}.git",
    config.ssh_host, config.ssh_port, scenario.project_path
  );
  let outsider_url = format!(
    "ssh://git@{}:{}/{}.git",
    config.ssh_host, config.ssh_port, scenario.outsider_project_path
  );
  let master = session::Master::start(config, &key, &credential.known_hosts)?;
  master.git(
    &personal_work,
    ["ls-remote", &url],
    true,
    "personal ControlMaster initial read",
  )?;
  master.git(
    &personal_work,
    ["ls-remote", &outsider_url],
    false,
    "personal SSH private project isolation",
  )?;
  let wrong_user = url.replacen("git@", "wrong-user@", 1);
  if commands::git_status(
    &personal_work,
    &key,
    &credential.known_hosts,
    ["ls-remote", &wrong_user],
  )? {
    bail!("personal SSH accepted a username other than git");
  }
  let first = personal_work.join("clone-one");
  let second = personal_work.join("clone-two");
  for clone in [&first, &second] {
    commands::run_git(
      &personal_work,
      &key,
      &credential.known_hosts,
      [
        "clone",
        &url,
        clone.to_str().context("personal clone path UTF-8")?,
      ],
      "personal SSH clone",
    )?;
  }
  let branch = format!("personal-{}", scenario.suffix);
  commands::run_git(
    &first,
    &key,
    &credential.known_hosts,
    ["checkout", "-b", &branch],
    "create personal SSH feature branch",
  )?;
  std::fs::write(
    first.join("personal-ssh.txt"),
    b"personal SSH clone push pull\n",
  )?;
  session::commit(&first, &key, &credential.known_hosts)?;
  master.git(
    &first,
    ["push", "--set-upstream", "origin", &branch],
    true,
    "ordinary developer personal SSH push",
  )?;
  commands::run_git(
    &second,
    &key,
    &credential.known_hosts,
    ["pull", "--ff-only", "origin", &branch],
    "personal SSH pull",
  )?;
  if std::fs::read(second.join("personal-ssh.txt"))? != b"personal SSH clone push pull\n" {
    bail!("personal SSH pull did not receive exact pushed bytes");
  }
  reporter.pass("personal-ssh-clone-push-pull","ordinary non-admin developer exchanged exact bytes through pinned real SSH clients; private project and SSH username isolation passed");
  policy::run(&client, config, scenario, &account, &master, &first)?;
  api::status(&client, config, scenario, &account, "disabled")?;
  master.git(
    &first,
    ["ls-remote", "origin"],
    false,
    "disabled creator denied on existing SSH connection",
  )?;
  api::status(&client, config, scenario, &account, "active")?;
  master.git(
    &first,
    ["ls-remote", "origin"],
    true,
    "reactivated creator regains personal SSH read",
  )?;
  api::login(&client, config, &mut account, reporter)?;
  reporter.pass("personal-ssh-live-roles-and-protection","existing SSH connection obeyed maintainer promotion, reporter downgrade, member removal/restoration, protected min-role, no-force, no-delete and require-MR policies");
  let keys = api::ok(
    &client,
    config,
    Method::GET,
    "/users/me/ssh-keys",
    Some(&account.token),
    None,
    "list used personal SSH key",
  )?;
  let used = keys
    .as_array()
    .and_then(|keys| keys.iter().find(|v| v["id"] == key_id))
    .context("personal SSH list omitted owned key")?;
  if used["last_used_at"].as_str().is_none() {
    bail!("successful personal SSH command did not record last_used_at");
  }
  api::ok(
    &client,
    config,
    Method::DELETE,
    &key_path,
    Some(&account.token),
    None,
    "revoke personal SSH key",
  )?;
  master.git(
    &first,
    ["ls-remote", "origin"],
    false,
    "revoked personal key denied on already authenticated connection",
  )?;
  if commands::git_status(
    &personal_work,
    &key,
    &credential.known_hosts,
    ["ls-remote", &url],
  )? {
    bail!("revoked personal SSH key authenticated a fresh connection");
  }
  reporter.pass("personal-ssh-existing-connection-revocation","revocation denied a new command on the still-live ControlMaster and denied fresh SSH authentication; cross-account/private-key/cross-mode registration checks passed");
  Ok(())
}
