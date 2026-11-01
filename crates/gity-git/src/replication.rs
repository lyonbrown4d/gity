use crate::{GitProcess, GitService};
use anyhow::{ensure, Context, Result};
use git2::{Oid, Repository};
use gity_git_wire::{ReferenceSnapshot, RepositoryHead};
use std::collections::BTreeMap;

impl GitService {
  /// Capture direct refs and the repository's symbolic or detached HEAD.
  pub async fn reference_snapshot(&self, repo: &str) -> Result<ReferenceSnapshot> {
    let path = self.repository_path(repo)?;
    tokio::task::spawn_blocking(move || {
      let repository = Repository::open_bare(path).context("open bare Git repository")?;
      let head = repository.find_reference("HEAD")?;
      let head = match head.symbolic_target()? {
        Some(target) => {
          ensure!(
            git2::Reference::is_valid_name(target),
            "invalid symbolic HEAD"
          );
          RepositoryHead::Symbolic(target.to_owned())
        }
        None => RepositoryHead::Detached(
          head
            .target()
            .context("detached HEAD has no target")?
            .to_string(),
        ),
      };

      let mut refs = BTreeMap::new();
      for reference in repository.references()? {
        let reference = reference?;
        let name = reference
          .name()
          .context("repository contains a non-UTF-8 reference name")?;
        if let Some(target) = reference.target() {
          refs.insert(name.to_owned(), target.to_string());
        }
      }
      Ok(ReferenceSnapshot { head, refs })
    })
    .await
    .context("repository snapshot worker failed")?
  }

  /// Stream a complete v3 bundle to stdout. Empty repositories have no bundle.
  pub async fn spawn_bundle(&self, repo: &str) -> Result<GitProcess> {
    let snapshot = self.reference_snapshot(repo).await?;
    ensure!(
      !snapshot.refs.is_empty(),
      "cannot bundle a repository without refs"
    );
    let permit = self.rpc_limit.clone().acquire_owned().await?;
    let mut command = self.command(repo)?;
    command.args(["bundle", "create", "--version=3", "-", "--all"]);
    crate::protocol::spawn_process(command, permit)
  }
}

pub(crate) fn parse_oid(value: &str) -> Result<Oid> {
  let oid = Oid::from_str(value).context("invalid Git object ID")?;
  anyhow::ensure!(oid.to_string() == value, "non-canonical Git object ID");
  Ok(oid)
}

impl GitService {
  /// Verify native object hashes and connectivity before exposing a promoted replica.
  pub async fn verify_integrity(&self, repo: &str) -> anyhow::Result<()> {
    let permit = self.rpc_limit.clone().acquire_owned().await?;
    let mut command = self.command(repo)?;
    command.args(["fsck", "--full", "--no-reflogs"]);
    let mut reader = crate::protocol::spawn_process(command, permit)?.into_reader()?;
    tokio::io::copy(&mut reader, &mut tokio::io::sink()).await?;
    Ok(())
  }
}
