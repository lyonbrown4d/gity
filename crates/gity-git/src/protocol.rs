use crate::{GitProcess, GitService};
use anyhow::{bail, Context, Result};
use std::{collections::BTreeMap, process::Stdio};
use tokio::{io::AsyncReadExt, process::Command, sync::OwnedSemaphorePermit};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitMaintenance {
  GarbageCollect,
  Repack,
  UpdateServerInfo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitRpc {
  UploadPack,
  ReceivePack,
}
impl GitRpc {
  pub(crate) fn command(self) -> &'static str {
    match self {
      Self::UploadPack => "upload-pack",
      Self::ReceivePack => "receive-pack",
    }
  }
}

impl GitService {
  pub async fn maintenance(&self, repo: &str, operation: GitMaintenance) -> Result<()> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::Maintenance {
            kind: match operation {
              GitMaintenance::GarbageCollect => gity_git_wire::Maintenance::GarbageCollect,
              GitMaintenance::Repack => gity_git_wire::Maintenance::Repack,
              GitMaintenance::UpdateServerInfo => gity_git_wire::Maintenance::UpdateServerInfo,
            },
          },
        )
        .await;
    }
    let args: &[&str] = match operation {
      GitMaintenance::GarbageCollect => &["gc", "--auto"],
      GitMaintenance::Repack => &["repack", "-a", "-d"],
      GitMaintenance::UpdateServerInfo => &["update-server-info"],
    };
    let output = self.command(repo)?.args(args).output().await?;
    if !output.status.success() {
      bail!(
        "Git maintenance failed: {}",
        String::from_utf8_lossy(&output.stderr)
      );
    }
    Ok(())
  }

  pub(crate) fn command(&self, repo: &str) -> Result<Command> {
    let mut command = Command::new(self.git_bin.as_ref());
    #[cfg(unix)]
    command.process_group(0);
    command
      .current_dir(self.repository_path(repo)?)
      .kill_on_drop(true)
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped());
    // Process-level ambient Git overrides must not select a different repository.
    for key in [
      "GIT_DIR",
      "GIT_WORK_TREE",
      "GIT_COMMON_DIR",
      "GIT_INDEX_FILE",
      "GIT_OBJECT_DIRECTORY",
      "GIT_ALTERNATE_OBJECT_DIRECTORIES",
      "GIT_CONFIG_COUNT",
    ] {
      command.env_remove(key);
    }
    command
      .env("GIT_TERMINAL_PROMPT", "0")
      .env("GIT_CONFIG_NOSYSTEM", "1");
    Ok(command)
  }

  pub async fn spawn_rpc(
    &self,
    repo: &str,
    service: GitRpc,
    advertise: bool,
    env: BTreeMap<String, String>,
  ) -> Result<GitProcess> {
    if self.remote.is_some() {
      return self
        .stream_remote(
          repo,
          gity_git_wire::TransferKind::Http {
            receive: service == GitRpc::ReceivePack,
            advertise,
            environment: env,
          },
          service == GitRpc::ReceivePack && !advertise,
        )
        .await;
    }
    let permit = self.rpc_limit.clone().acquire_owned().await?;
    if service == GitRpc::ReceivePack && !advertise {
      // Never silently bypass branch protection because the adapter is absent.
      let hook = self.repository_path(repo)?.join("hooks/update");
      let metadata =
        std::fs::metadata(&hook).context("receive-pack requires an installed update hook")?;
      if !metadata.is_file() {
        bail!("receive-pack update hook is not a file");
      }
      #[cfg(unix)]
      {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
          bail!("receive-pack update hook is not executable");
        }
      }
    }
    let mut command = self.command(repo)?;
    command
      .arg("-c")
      .arg("transfer.hideRefs=refs/gity/")
      .arg(service.command())
      .arg("--stateless-rpc");
    if advertise {
      command.arg("--advertise-refs");
    } else {
      command.stdin(Stdio::piped());
    }
    command.arg(".");
    for (key, value) in env {
      if !matches!(
        key.as_str(),
        "GIT_PROTOCOL"
          | "GITY_DENY_FORCE_PUSH_REFS"
          | "GITY_DENY_DELETE_REFS"
          | "GITY_DENY_UPDATE_REFS"
      ) {
        bail!("unsupported Git request environment variable");
      }
      command.env(key, value);
    }
    spawn_process(command, permit)
  }
}

pub(crate) fn spawn_process(
  mut command: Command,
  permit: OwnedSemaphorePermit,
) -> Result<GitProcess> {
  let mut child = command.spawn().context("start Git protocol process")?;
  let mut stderr = child.stderr.take().context("capture Git stderr")?;
  let task = tokio::spawn(async move {
    let mut kept = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
      let count = stderr.read(&mut buffer).await?;
      if count == 0 {
        break;
      }
      let retain = count.min(65536usize.saturating_sub(kept.len()));
      kept.extend_from_slice(&buffer[..retain]);
    }
    Ok(kept)
  });
  Ok(GitProcess::local(child, task, permit))
}
