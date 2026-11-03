pub(crate) use crate::references::validate_ref;
use crate::{commit, validate_path, GitService};
#[path = "file_commit.rs"]
mod file_commit;
#[path = "initializer.rs"]
mod initializer;
use anyhow::{bail, Context, Result};
use git2::{Repository, Signature};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Deserialize, Serialize)]
pub struct CreateFileCommitInput {
  pub branch_name: String,
  pub file_path: String,
  pub content: String,
  pub message: String,
  /// Expected branch tip; an empty string explicitly expects an absent branch.
  #[serde(default)]
  pub expected_head_sha: Option<String>,
  /// Only updates an existing blob; requires a nonempty expected branch tip.
  #[serde(default)]
  pub update_existing: bool,
  #[serde(default)]
  pub author_name: String,
  #[serde(default)]
  pub author_email: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MergeBranchesInput {
  pub target_branch: String,
  pub source_branch: String,
  #[serde(default)]
  pub expected_source_sha: Option<String>,
  #[serde(default)]
  pub message: String,
  #[serde(default)]
  pub author_name: String,
  #[serde(default)]
  pub author_email: String,
}

impl GitService {
  pub fn validate_initial_branch(&self, branch: &str) -> Result<()> {
    validate_ref(branch, "heads").map(|_| ())
  }

  pub fn validate_initial_commit(
    &self,
    branch: &str,
    initial_commit: Option<&gity_git_wire::InitialCommit>,
  ) -> Result<()> {
    initializer::validate(branch, initial_commit)
  }

  pub async fn init_bare(&self, repo: &str, initial_branch: &str) -> Result<()> {
    self.init_bare_with_commit(repo, initial_branch, None).await
  }

  pub async fn init_bare_with_commit(
    &self,
    repo: &str,
    initial_branch: &str,
    initial_commit: Option<gity_git_wire::InitialCommit>,
  ) -> Result<()> {
    self.validate_initial_commit(initial_branch, initial_commit.as_ref())?;
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::InitBare {
            initial_branch: initial_branch.into(),
            initial_commit,
          },
        )
        .await;
    }
    let (path, branch, initial_commit) = (
      self.repository_path(repo)?,
      initial_branch.to_owned(),
      initial_commit,
    );
    tokio::task::spawn_blocking(move || -> Result<()> {
      if path.exists() {
        bail!("repository path already exists");
      }
      let mut opts = git2::RepositoryInitOptions::new();
      opts
        .bare(true)
        .initial_head(&branch)
        .mkdir(true)
        .mkpath(true);
      let repository = Repository::init_opts(path, &opts)?;
      initializer::initialize(&repository, &branch, initial_commit.as_ref())?;
      Ok(())
    })
    .await??;
    Ok(())
  }

  pub async fn resume_bare_initialization(&self, repo: &str, initial_branch: &str) -> Result<()> {
    self
      .resume_bare_initialization_with_commit(repo, initial_branch, None)
      .await
  }

  pub async fn resume_bare_initialization_with_commit(
    &self,
    repo: &str,
    initial_branch: &str,
    initial_commit: Option<gity_git_wire::InitialCommit>,
  ) -> Result<()> {
    initializer::validate(initial_branch, initial_commit.as_ref())?;
    validate_ref(initial_branch, "heads")?;
    let (path, branch, initial_commit) = (
      self.repository_path(repo)?,
      initial_branch.to_owned(),
      initial_commit,
    );
    tokio::task::spawn_blocking(move || -> Result<()> {
      let metadata = std::fs::symlink_metadata(&path)?;
      anyhow::ensure!(
        metadata.is_dir() && !metadata.file_type().is_symlink(),
        "initializing repository path must be a directory"
      );
      let mut opts = git2::RepositoryInitOptions::new();
      opts.bare(true).initial_head(&branch);
      let repository = Repository::init_opts(path, &opts)?;
      initializer::initialize(&repository, &branch, initial_commit.as_ref())?;
      Ok(())
    })
    .await??;
    Ok(())
  }

  pub async fn create_branch(&self, repo: &str, name: &str, source: &str) -> Result<()> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::CreateBranch {
            name: name.into(),
            source: source.into(),
          },
        )
        .await;
    }
    let name = validate_ref(name, "heads")?;
    let source = source.to_owned();
    self
      .read(repo, move |r| {
        let c = commit(&r, &source, "")?;
        if r.find_reference(&name).is_ok() {
          bail!("git branch already exists");
        }
        r.reference(&name, c.id(), false, "create branch")?;
        Ok(Value::Null)
      })
      .await?;
    Ok(())
  }

  pub async fn delete_branch(&self, repo: &str, name: &str) -> Result<()> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::DeleteBranch { name: name.into() },
        )
        .await;
    }
    self.delete_ref(repo, name, "heads").await
  }
  pub async fn delete_tag(&self, repo: &str, name: &str) -> Result<()> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::DeleteTag { name: name.into() },
        )
        .await;
    }
    self.delete_ref(repo, name, "tags").await
  }
  async fn delete_ref(&self, repo: &str, name: &str, kind: &str) -> Result<()> {
    let reference = validate_ref(name, kind)?;
    self
      .read(repo, move |r| {
        r.find_reference(&reference)
          .context("git reference not found")?
          .delete()?;
        Ok(Value::Null)
      })
      .await?;
    Ok(())
  }

  pub async fn create_tag(&self, repo: &str, name: &str, source: &str) -> Result<()> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::CreateTag {
            name: name.into(),
            source: source.into(),
          },
        )
        .await;
    }
    let name = validate_ref(name, "tags")?;
    let source = source.to_owned();
    self
      .read(repo, move |r| {
        if r.find_reference(&name).is_ok() {
          bail!("git tag already exists");
        }
        r.reference(&name, commit(&r, &source, "")?.id(), false, "create tag")?;
        Ok(Value::Null)
      })
      .await?;
    Ok(())
  }

  /// Build objects without a working tree and compare-and-swap the branch tip.
  /// Competing pushes cannot silently overwrite commits made by another writer.
  pub async fn create_file_commit(
    &self,
    repo: &str,
    input: CreateFileCommitInput,
  ) -> Result<String> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::CreateFileCommit {
            input: serde_json::to_value(&input)?,
          },
        )
        .await;
    }
    let reference = validate_ref(&input.branch_name, "heads")?;
    validate_path(&input.file_path, false)?;
    if input.message.trim().is_empty() {
      bail!("commit message is required");
    }
    let result = self
      .read(repo, move |r| file_commit::create(&r, &reference, &input))
      .await?;
    Ok(
      result["hash"]
        .as_str()
        .context("commit result lacks object ID")?
        .to_owned(),
    )
  }
}

pub(crate) fn signature(name: &str, email: &str) -> Result<Signature<'static>> {
  Ok(Signature::now(
    if name.trim().is_empty() { "Gity" } else { name },
    if email.trim().is_empty() {
      "noreply@gity.local"
    } else {
      email
    },
  )?)
}
