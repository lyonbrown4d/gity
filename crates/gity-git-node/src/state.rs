use crate::GitNodeService;
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
  fs::{self, File, OpenOptions},
  io::{Read, Write},
  path::PathBuf,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RepositoryRole {
  Primary,
  Replica,
  Fenced,
  InitializingPrimary,
  InitializingReplica,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RepositoryState {
  pub role: RepositoryRole,
  pub epoch: u64,
  #[serde(default)]
  pub initial_branch: Option<String>,
}

impl GitNodeService {
  pub(crate) fn require_repository_state(
    &self,
    repo: &str,
    epoch: u64,
    allow_fenced: bool,
  ) -> Result<RepositoryState> {
    let state = self
      .load_repository_state(repo)?
      .context("repository has no durable ownership state")?;
    ensure!(epoch == state.epoch, "stale repository epoch");
    ensure!(
      matches!(
        state.role,
        RepositoryRole::Primary | RepositoryRole::Replica | RepositoryRole::Fenced
      ),
      "repository initialization is incomplete"
    );
    if state.role == RepositoryRole::Fenced && !allow_fenced {
      bail!("repository is fenced");
    }
    Ok(state)
  }

  pub(crate) fn load_repository_state(&self, repo: &str) -> Result<Option<RepositoryState>> {
    let path = self.state_file(repo)?;
    match fs::symlink_metadata(&path) {
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
      Err(error) => Err(error.into()),
      Ok(metadata) => {
        ensure!(metadata.is_file(), "repository state is not a regular file");
        ensure!(metadata.len() <= 4096, "repository state file is oversized");
        let mut file = OpenOptions::new().read(true).open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let state: RepositoryState =
          serde_json::from_slice(&bytes).context("repository state file is invalid")?;
        ensure!(state.epoch > 0, "repository epoch must be positive");
        Ok(Some(state))
      }
    }
  }

  pub(crate) fn save_repository_state(&self, repo: &str, state: RepositoryState) -> Result<()> {
    ensure!(state.epoch > 0, "repository epoch must be positive");
    let destination = self.state_file(repo)?;
    let directory = destination.parent().context("state directory is missing")?;
    if let Ok(metadata) = fs::symlink_metadata(&destination) {
      ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "repository state file must not be a symlink"
      );
    }
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    serde_json::to_writer(&mut temporary, &state)?;
    temporary.write_all(b"\n")?;
    temporary.as_file().sync_all()?;
    temporary.persist(&destination)?;
    File::open(directory)?.sync_all()?;
    Ok(())
  }

  fn state_file(&self, repo: &str) -> Result<PathBuf> {
    let project_id = repo
      .strip_suffix(".git")
      .context("invalid canonical repository key")?;
    let parsed: u64 = project_id.parse().context("invalid project identifier")?;
    ensure!(
      parsed > 0 && parsed.to_string() == project_id,
      "invalid project identifier"
    );

    let root = self.settings.repo_root.canonicalize()?;
    let directory = root.join(".gity-node-state");
    let created = match fs::symlink_metadata(&directory) {
      Ok(_) => false,
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
        match fs::create_dir(&directory) {
          Ok(()) => true,
          Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => false,
          Err(error) => return Err(error.into()),
        }
      }
      Err(error) => return Err(error.into()),
    };
    let metadata = fs::symlink_metadata(&directory)?;
    ensure!(
      metadata.is_dir() && !metadata.file_type().is_symlink(),
      "repository state directory must not be a symlink"
    );
    if created {
      #[cfg(unix)]
      {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
      }
      File::open(&root)?.sync_all()?;
    }
    let canonical_directory = directory.canonicalize()?;
    ensure!(
      canonical_directory.starts_with(&root),
      "repository state directory escapes repository root"
    );
    Ok(canonical_directory.join(format!("{project_id}.json")))
  }
}
