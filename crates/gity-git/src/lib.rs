//! Repository operations use libgit2; the native Git process owns wire protocols.
//! Repository storage must be owned exclusively by the server operating-system user.
mod archive;
mod blob;
mod content;
mod hooks;
mod matcher;
mod merge;
mod mutations;
mod process;
mod protocol;
mod references;
mod remote;
mod remote_stream;
mod replication;
mod replication_apply;
mod search;
mod ssh;
mod ssh_policy;
pub use hooks::check_update;
pub use matcher::SearchPlan;
pub use mutations::{CreateFileCommitInput, MergeBranchesInput};
pub use process::{GitInput, GitOutput, GitProcess, GitReader};
pub use protocol::{GitMaintenance, GitRpc};
pub use search::{walk_blobs, SearchParams};
pub use ssh_policy::{SshBranchRule, SshPolicy, SshPushAccess, SshRuleType, SSH_POLICY_ENV};

use anyhow::{bail, Context, Result};
use git2::{Commit, Repository};
use serde_json::Value;
use std::{
  path::{Component, Path, PathBuf},
  sync::Arc,
};
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct GitService {
  remote: Option<Arc<dyn gity_git_wire::GitNodeRouter>>,
  root: Arc<PathBuf>,
  git_bin: Arc<PathBuf>,
  rpc_limit: Arc<Semaphore>,
}

impl GitService {
  pub fn new(root: impl Into<PathBuf>, git_bin: impl Into<PathBuf>) -> Self {
    Self {
      remote: None,
      root: Arc::new(root.into()),
      git_bin: Arc::new(git_bin.into()),
      rpc_limit: Arc::new(Semaphore::new(32)),
    }
  }

  /// Reject traversal and symlink escapes, including nonexistent creation targets.
  pub fn repository_path(&self, relative: &str) -> Result<PathBuf> {
    anyhow::ensure!(
      self.remote.is_none(),
      "remote repositories are owned by Git storage nodes"
    );
    validate_path(relative, false)?;
    let root = self
      .root
      .canonicalize()
      .context("resolve repository root")?;
    let target = root.join(relative);
    let mut ancestor = target.as_path();
    while !ancestor.exists() {
      // A dangling symlink is not an absent directory that we may create.
      if std::fs::symlink_metadata(ancestor).is_ok() {
        bail!("repository path contains a dangling symlink");
      }
      ancestor = ancestor.parent().context("invalid repository path")?;
    }
    if !ancestor.canonicalize()?.starts_with(&root) {
      bail!("repository path escapes repo root");
    }
    Ok(target)
  }

  async fn read<F>(&self, repo: &str, operation: F) -> Result<Value>
  where
    F: FnOnce(Repository) -> Result<Value> + Send + 'static,
  {
    let path = self.repository_path(repo)?;
    tokio::task::spawn_blocking(move || {
      let repo = Repository::open_bare(path).context("git repository not found")?;
      operation(repo)
    })
    .await
    .context("repository worker failed")?
  }
}

pub(crate) fn validate_path(value: &str, allow_empty: bool) -> Result<()> {
  if value.is_empty() && allow_empty {
    return Ok(());
  }
  if value.is_empty()
    || value.contains('\\')
    || value.contains('\0')
    || Path::new(value)
      .components()
      .any(|c| !matches!(c, Component::Normal(_)))
  {
    bail!("invalid repository or file path");
  }
  if value
    .split('/')
    .any(|p| p.eq_ignore_ascii_case(".git") || p.is_empty())
  {
    bail!("invalid repository or file path");
  }
  Ok(())
}

pub(crate) fn commit<'a>(
  repo: &'a Repository,
  revision: &str,
  default: &str,
) -> Result<Commit<'a>> {
  let candidates = if revision.trim().is_empty() {
    vec![format!("refs/heads/{default}"), "HEAD".to_owned()]
  } else {
    vec![
      format!("refs/heads/{revision}"),
      format!("refs/tags/{revision}"),
      revision.to_owned(),
    ]
  };
  for candidate in candidates {
    if let Ok(object) = repo.revparse_single(&candidate) {
      if let Ok(commit) = object.peel_to_commit() {
        return Ok(commit);
      }
    }
  }
  if is_empty(repo)? {
    bail!("git repository is empty");
  }
  bail!("git reference not found")
}

// libgit2's is_empty also compares HEAD with init.defaultBranch, so it
// misclassifies repositories initialized with an explicit different branch.
pub(crate) fn is_empty(repo: &Repository) -> Result<bool> {
  Ok(repo.references()?.next().transpose()?.is_none())
}

pub(crate) fn timestamp(seconds: i64) -> String {
  chrono::DateTime::from_timestamp(seconds, 0)
    .map(|t| t.format("%Y-%m-%dT%H:%M:%SZ").to_string())
    .unwrap_or_default()
}
