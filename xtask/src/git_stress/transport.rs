use super::{Config, Transport};
use anyhow::{Context, Result, bail};
use std::{
  fs,
  path::Path,
  process::{Command, Stdio},
};

pub(super) fn validate(config: &mut Config) -> Result<()> {
  let base = url::Url::parse(&config.base_url).context("invalid API base URL")?;
  if !matches!(base.scheme(), "http" | "https")
    || base.host_str().is_none()
    || !base.username().is_empty()
    || base.password().is_some()
  {
    bail!("API base URL must be HTTP(S) without embedded credentials");
  }
  if config.transport == Transport::Ssh {
    for path in [&mut config.ssh_key, &mut config.ssh_known_hosts] {
      let path = path
        .as_mut()
        .context("SSH requires --ssh-key and --ssh-known-hosts")?;
      if !fs::metadata(&*path)?.is_file() {
        bail!("SSH credentials must be regular files");
      }
      known_hosts_option(path)?; // Reject configuration-line injection before setup.
      *path = path.canonicalize()?;
    }
    project_url(config, "probe", "http://localhost/probe.git")?;
  }
  if !config.project_url.is_empty() {
    repository_url(&config.project_url, config)?;
  }
  Ok(())
}

pub(super) fn repository_url(raw: &str, config: &Config) -> Result<String> {
  let mut target = url::Url::parse(raw).context("invalid repository URL")?;
  if target.host_str().is_none() {
    bail!("repository URL requires a host");
  }
  match config.transport {
    Transport::Http => {
      if !matches!(target.scheme(), "http" | "https") {
        bail!("HTTP transport requires an HTTP(S) repository URL");
      }
      target
        .set_username("")
        .map_err(|_| anyhow::anyhow!("invalid repository URL username"))?;
      target
        .set_password(None)
        .map_err(|_| anyhow::anyhow!("invalid repository URL password"))?;
    }
    Transport::Ssh => {
      if target.scheme() != "ssh"
        || !["", "git"].contains(&target.username())
        || target.password().is_some()
      {
        bail!("SSH transport requires ssh://git@host/path.git without a password");
      }
      target
        .set_username("git")
        .map_err(|_| anyhow::anyhow!("invalid SSH username"))?;
      if target.port().is_none() {
        target
          .set_port(Some(config.ssh_port))
          .map_err(|_| anyhow::anyhow!("invalid SSH port"))?;
      }
    }
  }
  target.set_query(None);
  target.set_fragment(None);
  Ok(target.to_string())
}

pub(super) fn project_url(config: &Config, project_path: &str, http_url: &str) -> Result<String> {
  if config.transport == Transport::Http {
    return repository_url(http_url, config);
  }
  let base = url::Url::parse(&config.base_url)?;
  let host = config
    .ssh_host
    .clone()
    .or_else(|| base.host().map(|h| h.to_string()))
    .context("SSH host missing")?;
  let mut target = url::Url::parse(&format!("ssh://git@{host}:{}", config.ssh_port))
    .context("invalid SSH host")?;
  target.set_path(&format!("/{project_path}.git"));
  repository_url(target.as_str(), config)
}

pub(super) fn public_key(path: &Path) -> Result<String> {
  let result = Command::new("ssh-keygen")
    .args(["-y", "-P", "", "-f"])
    .arg(path)
    .stdin(Stdio::null())
    .output()
    .context("derive supplied SSH key's public material")?;
  if !result.status.success() || result.stdout.len() > 16384 {
    bail!("cannot derive supplied SSH public key; use a readable noninteractive benchmark key");
  }
  Ok(
    String::from_utf8(result.stdout)
      .context("SSH public material is not UTF-8")?
      .trim()
      .into(),
  )
}

/// Git executes a fixed wrapper whose pathname is quoted with shlex.
/// Credential paths travel only as separately quoted environment arguments.
pub(super) const SSH_WRAPPER: &str = "#!/bin/sh\nexec ssh -F /dev/null -i \"$GITY_GIT_PERF_SSH_KEY\" -o \"$GITY_GIT_PERF_KNOWN_HOSTS_OPTION\" -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=yes -o GlobalKnownHostsFile=/dev/null -o UpdateHostKeys=no -o PasswordAuthentication=no -o KbdInteractiveAuthentication=no -o ConnectTimeout=8 \"$@\"\n";

pub(super) fn known_hosts_option(path: &Path) -> Result<String> {
  let value = path.to_str().context("SSH path is not UTF-8")?;
  if value.contains(['\0', '\r', '\n']) {
    bail!("SSH paths may not contain control line breaks");
  }
  Ok(format!(
    "UserKnownHostsFile=\"{}\"",
    value
      .replace('\\', "\\\\")
      .replace('"', "\\\"")
      .replace('%', "%%")
  ))
}

#[cfg(test)]
#[path = "transport_tests.rs"]
mod tests;
