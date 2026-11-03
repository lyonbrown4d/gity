use crate::{
  GitNodeService,
  state::{RepositoryRole, RepositoryState},
};
use anyhow::{Context, Result, ensure};
use gity_git_wire::InitialCommit;
use gity_git_wire::ReferenceSnapshot;
use serde_json::Value;
use sha2::{Digest, Sha256};

impl GitNodeService {
  pub(crate) async fn initialize_primary(
    &self,
    repo: &str,
    request_epoch: u64,
    initial_branch: &str,
    initial_commit: Option<InitialCommit>,
  ) -> Result<Value> {
    ensure!(request_epoch == 1, "new repositories must start at epoch 1");
    self
      .git
      .validate_initial_commit(initial_branch, initial_commit.as_ref())?;
    let fingerprint = initial_commit
      .as_ref()
      .map(|commit| commit_fingerprint(commit))
      .transpose()?;
    self.allow_write().map_err(anyhow::Error::msg)?;
    match self.load_repository_state(repo)? {
      None => {
        let path = self.git.repository_path(repo)?;
        ensure!(!path.exists(), "unmanaged repository path already exists");
        self.save_repository_state(
          repo,
          RepositoryState {
            role: RepositoryRole::InitializingPrimary,
            epoch: 1,
            initial_branch: Some(initial_branch.to_owned()),
            initial_commit_sha256: fingerprint.clone(),
          },
        )?;
        self
          .git
          .init_bare_with_commit(repo, initial_branch, initial_commit.clone())
          .await?;
      }
      Some(state) if state.role == RepositoryRole::InitializingPrimary => {
        ensure!(state.epoch == 1, "stale primary initialization epoch");
        ensure!(
          state.initial_branch.as_deref() == Some(initial_branch),
          "primary initialization branch does not match its durable marker"
        );
        ensure!(
          state.initial_commit_sha256 == fingerprint,
          "primary initialization files do not match their durable marker"
        );
        let path = self.git.repository_path(repo)?;
        if path.exists() {
          self
            .git
            .resume_bare_initialization_with_commit(repo, initial_branch, initial_commit.clone())
            .await?;
        } else {
          self
            .git
            .init_bare_with_commit(repo, initial_branch, initial_commit.clone())
            .await?;
        }
      }
      Some(_) => anyhow::bail!("repository is already managed"),
    }
    self
      .git
      .ensure_update_hook(repo, &self.settings.hook_executable)
      .await?;
    self.save_repository_state(
      repo,
      RepositoryState {
        role: RepositoryRole::Primary,
        epoch: 1,
        initial_branch: None,
        initial_commit_sha256: None,
      },
    )?;
    Ok(Value::Null)
  }

  pub(crate) async fn fence_repository(
    &self,
    repo: &str,
    request_epoch: u64,
    next_epoch: u64,
  ) -> Result<ReferenceSnapshot> {
    let state = self
      .load_repository_state(repo)?
      .context("cannot fence an unmanaged repository")?;
    ensure!(
      matches!(
        state.role,
        RepositoryRole::Primary | RepositoryRole::Replica | RepositoryRole::Fenced
      ),
      "cannot fence a repository during initialization"
    );
    if state.role == RepositoryRole::Fenced
      && state.epoch == next_epoch
      && request_epoch <= state.epoch
    {
      return self.git.reference_snapshot(repo).await;
    }
    ensure!(request_epoch == state.epoch, "stale repository epoch");
    ensure!(next_epoch > state.epoch, "fencing epoch must increase");
    let snapshot = self.git.reference_snapshot(repo).await?;
    self.save_repository_state(
      repo,
      RepositoryState {
        role: RepositoryRole::Fenced,
        epoch: next_epoch,
        initial_branch: None,
        initial_commit_sha256: None,
      },
    )?;
    Ok(snapshot)
  }

  pub(crate) async fn activate_primary(
    &self,
    repo: &str,
    request_epoch: u64,
    epoch: u64,
  ) -> Result<ReferenceSnapshot> {
    ensure!(
      epoch > 0 && request_epoch == epoch,
      "activation epoch mismatch"
    );
    let current = self
      .load_repository_state(repo)?
      .context("cannot activate an unmanaged repository")?;
    ensure!(
      matches!(
        current.role,
        RepositoryRole::Primary | RepositoryRole::Replica | RepositoryRole::Fenced
      ),
      "cannot activate a repository during initialization"
    );
    ensure!(epoch >= current.epoch, "cannot lower repository epoch");
    self.git.verify_integrity(repo).await?;
    let snapshot = self.git.reference_snapshot(repo).await?;
    self
      .git
      .ensure_update_hook(repo, &self.settings.hook_executable)
      .await?;
    self.save_repository_state(
      repo,
      RepositoryState {
        role: RepositoryRole::Primary,
        epoch,
        initial_branch: None,
        initial_commit_sha256: None,
      },
    )?;
    Ok(snapshot)
  }
}

fn commit_fingerprint(commit: &InitialCommit) -> Result<String> {
  let encoded = serde_json::to_vec(commit)?;
  Ok(format!("{:x}", Sha256::digest(encoded)))
}
