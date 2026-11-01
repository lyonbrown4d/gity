use super::{BlackboxArgs, Mode, Reporter, RunConfig, api, process};
use anyhow::{Context, Result, bail};
use reqwest::Method;
use serde_json::json;
use std::{
  path::{Path, PathBuf},
  process::Command,
};

mod commands;
mod http;
mod promotion;
pub(super) use promotion::after_promotion;

pub struct GitCredential {
  private_key: PathBuf,
  known_hosts: PathBuf,
}

pub fn run(
  root: &Path,
  args: &BlackboxArgs,
  config: &RunConfig,
  scenario: &api::Scenario,
  work: &Path,
  reporter: &mut Reporter,
) -> Result<GitCredential> {
  let key = work.join("ssh-owner-ed25519");
  let known_hosts = work.join("ssh-known-hosts");
  let public = create_key(&key)?;
  let api_client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
  let (status, _) = api::request_json(
    &api_client,
    config,
    Method::POST,
    &format!("/projects/{}/deploy-keys", scenario.project_id),
    Some(&scenario.owner_token),
    Some(json!({
      "title": format!("blackbox-{}", scenario.suffix),
      "public_key": public,
      "can_push": true
    })),
  )?;
  api::require_status(status, reqwest::StatusCode::OK, "SSH deploy key creation")?;

  let host_public = host_key(root, args, &scenario.project_path)?;
  let host = if config.ssh_port == 22 {
    config.ssh_host.clone()
  } else {
    format!("[{}]:{}", config.ssh_host, config.ssh_port)
  };
  std::fs::write(&known_hosts, format!("{host} {host_public}\n"))
    .context("write pinned SSH host key")?;
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&known_hosts, std::fs::Permissions::from_mode(0o600))?;
  }
  let url = format!(
    "ssh://git@{}:{}/{scenario_path}.git",
    config.ssh_host,
    config.ssh_port,
    scenario_path = scenario.project_path
  );
  let first = work.join("git-clone-one");
  let second = work.join("git-clone-two");
  commands::run_git(
    work,
    &key,
    &known_hosts,
    [
      "clone",
      url.as_str(),
      first.to_str().context("clone path is not UTF-8")?,
    ],
    "SSH first clone",
  )?;
  commands::run_git(
    work,
    &key,
    &known_hosts,
    [
      "clone",
      url.as_str(),
      second.to_str().context("clone path is not UTF-8")?,
    ],
    "SSH second clone",
  )?;
  commands::configure(&first, &key, &known_hosts)?;
  commands::configure(&second, &key, &known_hosts)?;
  std::fs::write(
    first.join("ssh-blackbox.txt"),
    "pushed by the first clone\n",
  )?;
  commands::run_git(
    &first,
    &key,
    &known_hosts,
    ["add", "ssh-blackbox.txt"],
    "git add",
  )?;
  commands::run_git(
    &first,
    &key,
    &known_hosts,
    ["commit", "-m", "blackbox first push"],
    "git commit",
  )?;
  commands::run_git(
    &first,
    &key,
    &known_hosts,
    ["push", "origin", "main"],
    "SSH push",
  )?;
  commands::run_git(
    &second,
    &key,
    &known_hosts,
    ["pull", "--ff-only", "origin", "main"],
    "SSH pull",
  )?;
  let pulled = std::fs::read_to_string(second.join("ssh-blackbox.txt"))?;
  if pulled != "pushed by the first clone\n" {
    bail!("second SSH clone did not receive the pushed file");
  }
  http::run(config, scenario, work, reporter)?;
  commands::run_git(
    &first,
    &key,
    &known_hosts,
    ["pull", "--ff-only", "origin", "main"],
    "SSH synchronize HTTP Git commit",
  )?;

  let bad_key = work.join("ssh-unknown-ed25519");
  create_key(&bad_key)?;
  let rejected = commands::git_status(
    work,
    &bad_key,
    &known_hosts,
    [
      "clone",
      url.as_str(),
      work.join("bad-key-clone").to_str().unwrap(),
    ],
  )?;
  if rejected {
    bail!("SSH accepted a key that was not registered for the project");
  }

  let (protect_status, _) = api::request_json(
    &api_client,
    config,
    Method::POST,
    &format!(
      "/projects/{}/repository/branches/main/protect",
      scenario.project_id
    ),
    Some(&scenario.owner_token),
    Some(json!({
      "push_access_level":"maintainer",
      "merge_access_level":"maintainer",
      "require_merge_request":false,
      "require_pipeline_success":false,
      "allow_force_push":false,
      "allow_delete":false
    })),
  )?;
  api::require_status(
    protect_status,
    reqwest::StatusCode::OK,
    "protect default branch",
  )?;
  std::fs::write(
    first.join("protected-blackbox.txt"),
    "must not reach protected main\n",
  )?;
  commands::run_git(
    &first,
    &key,
    &known_hosts,
    ["add", "."],
    "git add protected fixture",
  )?;
  commands::run_git(
    &first,
    &key,
    &known_hosts,
    ["commit", "-m", "blackbox protected push"],
    "git commit protected fixture",
  )?;
  if commands::git_status(&first, &key, &known_hosts, ["push", "origin", "main"])? {
    bail!("SSH accepted a push to a branch protected for maintainers");
  }
  reporter.pass(
    "ssh-clone-push-pull-bad-key-protected-branch",
    "pinned host key, registered writer, unknown-key rejection, and protected push rejection passed",
  );
  Ok(GitCredential {
    private_key: key,
    known_hosts,
  })
}

fn create_key(path: &Path) -> Result<String> {
  process::run_output(
    Command::new("ssh-keygen")
      .args(["-q", "-t", "ed25519", "-N", "", "-f"])
      .arg(path),
    "ssh-keygen",
  )?;
  let public = process::run_output(
    Command::new("ssh-keygen").args(["-y", "-f"]).arg(path),
    "derive SSH public key",
  )?;
  String::from_utf8(public.stdout)
    .context("SSH public key is not UTF-8")
    .map(|value| value.trim().to_owned())
}

fn host_key(root: &Path, args: &BlackboxArgs, _project_path: &str) -> Result<String> {
  let service = if args.mode == Mode::Standalone {
    "gity"
  } else {
    "git-server"
  };
  let mut command = Command::new("docker");
  super::runtime::configure_compose(&mut command, root, args)?;
  command.args([
    "exec",
    "--no-TTY",
    service,
    "ssh-keygen",
    "-y",
    "-f",
    "/var/lib/gity/ssh_host_ed25519",
  ]);
  let output = process::run_output(&mut command, "read SSH server public host key")?;
  let public =
    String::from_utf8(output.stdout).context("SSH server public host key is not UTF-8")?;
  if !public.starts_with("ssh-") {
    bail!("SSH service returned an invalid host public key");
  }
  Ok(public.trim().to_owned())
}
