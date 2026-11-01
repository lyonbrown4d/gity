use crate::{
  GitNodeService,
  state::{RepositoryRole, RepositoryState},
};
use anyhow::{Context, Result, ensure};
use gity_git_wire::ReferenceSnapshot;
use serde_json::Value;

impl GitNodeService {
  pub(crate) async fn initialize_primary(
    &self,
    repo: &str,
    request_epoch: u64,
    initial_branch: &str,
  ) -> Result<Value> {
    ensure!(request_epoch == 1, "new repositories must start at epoch 1");
    self.git.validate_initial_branch(initial_branch)?;
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
          },
        )?;
        self.git.init_bare(repo, initial_branch).await?;
      }
      Some(state) if state.role == RepositoryRole::InitializingPrimary => {
        ensure!(state.epoch == 1, "stale primary initialization epoch");
        ensure!(
          state.initial_branch.as_deref() == Some(initial_branch),
          "primary initialization branch does not match its durable marker"
        );
        let path = self.git.repository_path(repo)?;
        if path.exists() {
          self
            .git
            .resume_bare_initialization(repo, initial_branch)
            .await?;
        } else {
          self.git.init_bare(repo, initial_branch).await?;
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
      },
    )?;
    Ok(snapshot)
  }
}
