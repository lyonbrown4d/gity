use super::super::{BlackboxArgs, Mode, Reporter, process};
use anyhow::{Context, Result, bail};
use std::{collections::BTreeMap, path::Path, process::Command};
use uuid::Uuid;

pub fn prepare_environment(root: &Path, args: &mut BlackboxArgs, work: &Path) -> Result<()> {
  let deployment = work.join("deployment");
  let tls = deployment.join("tls");
  let secrets = deployment.join("secrets");
  let runner_work = deployment.join("runner-work");
  std::fs::create_dir_all(&tls)?;
  std::fs::create_dir_all(&secrets)?;
  std::fs::create_dir_all(&runner_work)?;

  let env_path = deployment.join("compose.env");
  let mut content = if let Some(source) = &args.env_file {
    std::fs::read_to_string(source)
      .with_context(|| format!("read Compose env file {}", source.display()))?
  } else {
    String::new()
  };
  let mut env = BTreeMap::new();
  if let Some(source) = &args.env_file {
    for item in dotenvy::from_path_iter(source)
      .with_context(|| format!("parse Compose env file {}", source.display()))?
    {
      let (key, value) = item.context("parse Compose environment entry")?;
      env.insert(key, value);
    }
  }
  for key in [
    "POSTGRES_PASSWORD",
    "AUTHORITY_PEER_TOKEN",
    "GIT_NODE_1_PEER_TOKEN",
    "GIT_NODE_2_PEER_TOKEN",
    "SEAWEED_ACCESS_KEY_ID",
    "SEAWEED_SECRET_KEY",
  ] {
    env
      .entry(key.to_owned())
      .or_insert_with(|| Uuid::new_v4().simple().to_string());
  }
  if !content.ends_with('\n') {
    content.push('\n');
  }
  for (key, value) in &env {
    if !content
      .lines()
      .any(|line| line.trim_start().starts_with(&format!("{key}=")))
    {
      content.push_str(&format!("{key}={value}\n"));
    }
  }
  if let Some(proxy) = super::super::proxy::read()? {
    for key in ["GITY_OUTBOUND_HTTP_PROXY", "GITY_OUTBOUND_HTTPS_PROXY"] {
      content.push_str(&format!("{key}={}\n", proxy.as_str()));
      env.insert(key.to_owned(), proxy.as_str().to_owned());
    }
  }
  content.push_str(&format!(
    "GITY_TLS_DIR={}\nGITY_SECRETS_DIR={}\nGITY_RUNNER_HOST_WORK_ROOT={}\n",
    env_quote(&tls),
    env_quote(&secrets),
    env_quote(&runner_work)
  ));
  std::fs::write(&env_path, content).context("write private black-box Compose env file")?;
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&env_path, std::fs::Permissions::from_mode(0o600))?;
  }

  run_script(
    root,
    "deploy/components/generate-tls.sh",
    &env,
    &tls,
    &secrets,
  )?;
  run_script(
    root,
    "deploy/components/prepare-secrets.sh",
    &env,
    &tls,
    &secrets,
  )?;
  set_container_ownership(&tls, &secrets, &runner_work, args.mode)?;
  args.env_file = Some(env_path);
  Ok(())
}

pub fn remember_environment_secrets(path: Option<&Path>, reporter: &mut Reporter) -> Result<()> {
  let Some(path) = path else {
    return Ok(());
  };
  for entry in dotenvy::from_path_iter(path)
    .with_context(|| format!("read generated Compose environment {}", path.display()))?
  {
    let (key, value) = entry.context("parse generated Compose environment")?;
    let key = key.to_ascii_uppercase();
    if ["TOKEN", "SECRET", "PASSWORD", "ACCESS_KEY"]
      .iter()
      .any(|marker| key.contains(marker))
    {
      reporter.remember_secret(&value);
    }
  }
  Ok(())
}

fn run_script(
  root: &Path,
  script: &str,
  env: &BTreeMap<String, String>,
  tls: &Path,
  secrets: &Path,
) -> Result<()> {
  let mut command = Command::new("sh");
  command
    .current_dir(root)
    .arg(root.join(script))
    .envs(env)
    .env("GITY_TLS_DIR", tls)
    .env("GITY_SECRETS_DIR", secrets);
  process::run_status(&mut command, "prepare isolated TLS and Compose secrets")
}

fn env_quote(path: &Path) -> String {
  let value = path
    .to_string_lossy()
    .replace('\\', "\\\\")
    .replace('"', "\\\"");
  format!("\"{value}\"")
}

fn set_container_ownership(
  tls: &Path,
  secrets: &Path,
  runner_work: &Path,
  mode: Mode,
) -> Result<()> {
  let seaweed = seaweed_owner(mode, query_seaweed_identity)?;
  let mut command = Command::new("docker");
  command.args(["run", "--rm", "--user", "0:0", "--network", "none"]);
  for (host, container) in [
    (tls, "/run/tls"),
    (secrets, "/run/secrets"),
    (runner_work, "/runner-work"),
  ] {
    command
      .arg("--volume")
      .arg(format!("{}:{container}", host.display()));
  }
  command.args([
    "alpine:3.22",
    "sh",
    "-c",
    ASSET_PERMISSIONS,
    "gity-asset-permissions",
    if seaweed.is_some() {
      "distributed"
    } else {
      "standalone"
    },
  ]);
  if let Some((uid, gid)) = seaweed {
    command.arg(uid.to_string()).arg(gid.to_string());
  }
  process::run_status(&mut command, "set private deployment asset ownership")
}

// Numeric identities are passed as arguments. Never interpolate credentials,
// caller-controlled paths or command text into this constant shell program.
const ASSET_PERMISSIONS: &str = "set -eu; chown -R 10001:10001 /run/tls /run/secrets /runner-work; chmod 0755 /run/tls /run/secrets /runner-work; chmod 0600 /run/tls/*.key /run/secrets/git-node-*-token; chmod 0400 /run/secrets/seaweed-s3.json /run/secrets/angos-s3.toml /run/secrets/angos-mirror.toml; if [ \"$1\" = distributed ]; then chown \"$2:$3\" /run/secrets/seaweed-s3.json; fi";

fn query_seaweed_identity() -> Result<Vec<u8>> {
  let output = process::run_output(
    Command::new("docker").args([
      "run",
      "--rm",
      "--pull",
      "never",
      "--network",
      "none",
      "--read-only",
      "--cap-drop",
      "ALL",
      "--entrypoint",
      "sh",
      "chrislusf/seaweedfs:4.51",
      "-c",
      "set -eu; id -u seaweed; id -g seaweed",
    ]),
    "inspect official SeaweedFS runtime UID/GID",
  )?;
  Ok(output.stdout)
}

fn seaweed_owner(
  mode: Mode,
  query: impl FnOnce() -> Result<Vec<u8>>,
) -> Result<Option<(u32, u32)>> {
  if mode == Mode::Standalone {
    return Ok(None);
  }
  let bytes = query()?;
  let text = std::str::from_utf8(&bytes).context("SeaweedFS identity is not UTF-8")?;
  let lines = text.lines().collect::<Vec<_>>();
  if lines.len() != 2 {
    bail!("SeaweedFS identity must contain exactly one UID and GID");
  }
  let numeric = |value: &str| -> Result<u32> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
      bail!("SeaweedFS UID/GID must be decimal numbers");
    }
    value
      .parse::<u32>()
      .context("SeaweedFS UID/GID exceeds numeric range")
  };
  Ok(Some((numeric(lines[0])?, numeric(lines[1])?)))
}

#[cfg(test)]
#[path = "deployment_tests.rs"]
mod tests;
