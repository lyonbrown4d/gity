use crate::GitService;
use anyhow::{bail, Context, Result};
use git2::Oid;
use std::path::Path;
impl GitService {
    /// Install an adapter to the Rust hook executable. Only this constant shell
    /// adapter is interpreted; refs and policy values are passed as arguments/env.
    pub async fn ensure_update_hook(&self, repo: &str, executable: impl AsRef<Path>) -> Result<()> {
        let path = self.repository_path(repo)?.join("hooks/update");
        let executable = executable
            .as_ref()
            .canonicalize()
            .context("resolve Git hook executable")?;
        let executable = executable.to_str().context("Git hook path is not UTF-8")?;
        let quoted = format!("'{}'", executable.replace('\'', "'\\''"));
        let script = format!("#!/bin/sh\n# gity-rust-protection-v1\nexec {quoted} update \"$@\"\n");
        if path.exists() {
            let current = std::fs::read_to_string(&path)?;
            if !current.contains("gity-rust-protection-v1")
                && !current.contains("GITY_DENY_FORCE_PUSH_REFS")
            {
                bail!("refusing to overwrite an unmanaged Git update hook");
            }
        }
        if std::fs::read_to_string(&path).ok().as_deref() == Some(script.as_str()) {
            return Ok(());
        }
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let temp = path.with_extension(format!("{}.{}.tmp", std::process::id(), sequence));
        std::fs::create_dir_all(path.parent().context("Git hooks directory")?)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        use std::io::Write;
        file.write_all(script.as_bytes())?;
        drop(file);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o755))?;
        }
        std::fs::rename(temp, path)?;
        Ok(())
    }
}

/// Called by the hook binary (or server's `git-hook update` subcommand) from
/// Git's repository working directory. Native Git honors quarantine object env.
pub fn check_update(
    repo: &Path,
    git_bin: &Path,
    reference: &str,
    old: &str,
    new: &str,
) -> Result<()> {
    if !git2::Reference::is_valid_name(reference) {
        bail!("invalid ref update");
    }
    let old = Oid::from_str(old).context("invalid old object ID")?;
    let new = Oid::from_str(new).context("invalid new object ID")?;
    match std::env::var(crate::SSH_POLICY_ENV) {
        Ok(json) => crate::SshPolicy::from_environment(&json)?.check_update(
            repo,
            git_bin,
            reference,
            &old.to_string(),
            &new.to_string(),
        )?,
        Err(std::env::VarError::NotPresent) => {}
        Err(error) => return Err(error).context("invalid SSH policy environment"),
    }
    let matches = |key: &str| {
        std::env::var(key)
            .unwrap_or_default()
            .lines()
            .any(|line| line == reference)
    };
    if matches("GITY_DENY_UPDATE_REFS") {
        bail!("updates are not allowed for protected ref {reference}");
    }
    if new.is_zero() && matches("GITY_DENY_DELETE_REFS") {
        bail!("deletion is not allowed for protected ref {reference}");
    }
    if !old.is_zero() && !new.is_zero() && matches("GITY_DENY_FORCE_PUSH_REFS") {
        check_fast_forward(repo, git_bin, reference, old, new)?;
    }
    Ok(())
}

pub(crate) fn check_fast_forward(
    repo: &Path,
    git_bin: &Path,
    reference: &str,
    old: Oid,
    new: Oid,
) -> Result<()> {
    let output = std::process::Command::new(git_bin)
        .current_dir(repo)
        .args([
            "merge-base",
            "--is-ancestor",
            &old.to_string(),
            &new.to_string(),
        ])
        .output()
        .context("verify protected branch ancestry")?;
    if !output.status.success() {
        bail!("force push is not allowed for protected ref {reference}");
    }
    Ok(())
}
