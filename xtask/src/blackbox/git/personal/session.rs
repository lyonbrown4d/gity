use crate::blackbox::{RunConfig, process};
use anyhow::{Context, Result, bail};
use std::{
  path::{Path, PathBuf},
  process::{Child, Command, Stdio},
  time::{Duration, Instant},
};

/// A real OpenSSH connection kept alive across role changes and key revocation.
pub(super) struct Master {
  child: Child,
  _socket_dir: tempfile::TempDir,
  socket: PathBuf,
  target: String,
  port: u16,
}
impl Master {
  pub fn start(config: &RunConfig, key: &Path, known_hosts: &Path) -> Result<Self> {
    // Unix control sockets have a short path limit, independent of TMPDIR.
    let socket_dir = tempfile::Builder::new()
      .prefix("gity-ssh-")
      .tempdir_in("/tmp")?;
    let socket = socket_dir.path().join("control");
    let target = format!("git@{}", config.ssh_host);
    let child = Command::new("ssh")
      .args(["-F", "/dev/null", "-M", "-N", "-S"])
      .arg(&socket)
      .args(["-p", &config.ssh_port.to_string(), "-i"])
      .arg(key)
      .args([
        "-o",
        "IdentitiesOnly=yes",
        "-o",
        "BatchMode=yes",
        "-o",
        "StrictHostKeyChecking=yes",
        "-o",
        "ConnectTimeout=8",
        "-o",
        "ControlPersist=no",
      ])
      .arg("-o")
      .arg(format!("UserKnownHostsFile={}", known_hosts.display()))
      .arg(&target)
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
      .context("start pinned personal SSH ControlMaster")?;
    let mut master = Self {
      child,
      _socket_dir: socket_dir,
      socket,
      target,
      port: config.ssh_port,
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
      if master.alive()? {
        return Ok(master);
      }
      if master.child.try_wait()?.is_some() || Instant::now() >= deadline {
        bail!("personal SSH ControlMaster did not become available");
      }
      std::thread::sleep(Duration::from_millis(50));
    }
  }
  pub fn alive(&self) -> Result<bool> {
    Ok(
      Command::new("ssh")
        .args(["-F", "/dev/null", "-S"])
        .arg(&self.socket)
        .args(["-p", &self.port.to_string(), "-O", "check"])
        .arg(&self.target)
        .output()
        .context("check existing SSH ControlMaster")?
        .status
        .success(),
    )
  }
  pub fn git<const N: usize>(
    &self,
    cwd: &Path,
    args: [&str; N],
    expected: bool,
    action: &str,
  ) -> Result<()> {
    // ProxyCommand=false makes a missing/broken control socket fail; OpenSSH must
    // reuse the already authenticated connection and cannot silently reconnect.
    let ssh = format!(
      "ssh -F /dev/null -S {} -o ControlMaster=no -o ProxyCommand=false -o BatchMode=yes -o StrictHostKeyChecking=yes",
      quote(&self.socket)
    );
    let mut command = Command::new("git");
    command
      .current_dir(cwd)
      .env("GIT_SSH_COMMAND", ssh)
      .env("GIT_TERMINAL_PROMPT", "0")
      .env("GIT_CONFIG_NOSYSTEM", "1")
      .args(args);
    let output = command.output().with_context(|| format!("run {action}"))?;
    if output.status.success() != expected {
      bail!("{action} returned an unexpected Git status");
    }
    if !self.alive()? {
      bail!("{action} disconnected the existing SSH ControlMaster");
    }
    Ok(())
  }
}
impl Drop for Master {
  fn drop(&mut self) {
    let _ = self.child.kill();
    let _ = self.child.wait();
  }
}
fn quote(path: &Path) -> String {
  format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}

pub(super) fn commit(cwd: &Path, key: &Path, known_hosts: &Path) -> Result<()> {
  super::super::commands::configure(cwd, key, known_hosts)?;
  for args in [
    ["add", "personal-ssh.txt"],
    ["commit", "--message=personal SSH blackbox"],
  ] {
    let mut command = Command::new("git");
    command.current_dir(cwd).args(args);
    process::run_status(&mut command, "commit personal SSH fixture")?;
  }
  Ok(())
}
