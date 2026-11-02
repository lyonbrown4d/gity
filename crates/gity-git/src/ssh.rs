use crate::{protocol::spawn_process, GitProcess, GitRpc, GitService, SshPolicy, SSH_POLICY_ENV};
use anyhow::{bail, Context, Result};
use std::{ffi::OsString, process::Stdio};

impl GitService {
  /// SSH is a stateful Git session: native Git advertises refs before reading
  /// the client's update commands. HTTP's stateless RPC stays separate.
  pub async fn spawn_ssh(
    &self,
    repo: &str,
    service: GitRpc,
    git_protocol: Option<&str>,
    policy: Option<&SshPolicy>,
  ) -> Result<GitProcess> {
    if self.remote.is_some() {
      return self
        .stream_remote(
          repo,
          gity_git_wire::TransferKind::Ssh {
            receive: service == GitRpc::ReceivePack,
            protocol: git_protocol.map(str::to_owned),
            policy: policy.map(serde_json::to_value).transpose()?,
          },
          service == GitRpc::ReceivePack,
        )
        .await;
    }
    let policy = if service == GitRpc::ReceivePack {
      let policy = policy.context("SSH receive-pack requires a branch policy")?;
      let hook = self.repository_path(repo)?.join("hooks/update");
      let metadata =
        std::fs::metadata(&hook).context("SSH receive-pack requires an installed update hook")?;
      if !metadata.is_file() {
        bail!("SSH receive-pack update hook is not a file");
      }
      #[cfg(unix)]
      {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
          bail!("SSH receive-pack update hook is not executable");
        }
      }
      if !std::fs::read_to_string(&hook)?.contains("gity-rust-protection-v1") {
        bail!("SSH receive-pack requires the managed protection hook");
      }
      Some(policy.to_environment()?)
    } else {
      None
    };
    if let Some(value) = git_protocol {
      if !matches!(value, "version=0" | "version=1" | "version=2") {
        bail!("unsupported SSH Git protocol negotiation");
      }
    }
    let permit = self.rpc_limit.clone().acquire_owned().await?;
    let mut command = self.command(repo)?;
    for key in [
      SSH_POLICY_ENV,
      "GITY_DENY_FORCE_PUSH_REFS",
      "GITY_DENY_DELETE_REFS",
      "GITY_DENY_UPDATE_REFS",
      "GIT_PROTOCOL",
      "GIT_NAMESPACE",
      "GIT_CONFIG",
      "GIT_CONFIG_PARAMETERS",
      "GIT_CONFIG_GLOBAL",
    ] {
      command.env_remove(key);
    }
    // Repository and ambient Git configuration cannot redirect the update
    // hook away from the managed adapter installed by the server.
    let mut hook_path = OsString::from("core.hooksPath=");
    hook_path.push(self.repository_path(repo)?.join("hooks"));
    command
      .arg("-c")
      .arg(hook_path)
      .arg("-c")
      .arg("receive.fsckObjects=true")
      .arg("-c")
      .arg("transfer.hideRefs=refs/gity/")
      .arg(service.command())
      .arg(".")
      .stdin(Stdio::piped());
    if let Some(protocol) = git_protocol {
      command.env("GIT_PROTOCOL", protocol);
    }
    if let Some(policy) = policy {
      command.env(SSH_POLICY_ENV, policy);
    }
    spawn_process(command, permit)
  }
}
