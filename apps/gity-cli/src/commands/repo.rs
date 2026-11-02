//! `gity repo clone` — clone a project through the URL the server advertises.
use crate::client::Options;
use anyhow::{Context, Result, bail};
use clap::Subcommand;
use std::path::PathBuf;
use std::process::Command;

#[derive(Subcommand)]
pub enum RepoCommand {
  /// Clone a project into a local directory
  Clone {
    /// Project id or `group/project` path
    project: String,
    /// Destination directory; defaults to the project name
    directory: Option<PathBuf>,
    /// Embed the personal token in the clone URL (leave the remote without
    /// credentials for long-lived checkouts)
    #[arg(long)]
    with_token: bool,
  },
}

pub async fn run(command: RepoCommand, options: &Options) -> Result<()> {
  match command {
    RepoCommand::Clone {
      project,
      directory,
      with_token,
    } => clone(options, project, directory, with_token).await,
  }
}

async fn clone(
  options: &Options,
  project: String,
  directory: Option<PathBuf>,
  with_token: bool,
) -> Result<()> {
  let client = options.client()?;
  let id = client.project_id(&project).await?;
  let detail = client.get(&format!("/projects/{id}")).await?;
  let url = detail["clone_http_url"]
    .as_str()
    .filter(|url| !url.is_empty())
    .ok_or_else(|| anyhow::anyhow!("project {project} does not advertise a clone URL"))?;
  let target = directory.unwrap_or_else(|| {
    PathBuf::from(
      detail["path_key"]
        .as_str()
        .filter(|name| !name.is_empty())
        .unwrap_or("project"),
    )
  });
  let url = if with_token {
    embed_token(url, &crate::config::resolve(options)?.token)?
  } else {
    url.to_owned()
  };
  println!("cloning {url} into {}", target.display());
  let status = Command::new("git")
    .arg("clone")
    .arg(&url)
    .arg(&target)
    .status()
    .context("run git clone; make sure git is installed")?;
  if !status.success() {
    bail!("git clone failed with {status}");
  }
  Ok(())
}

/// `http://host/path` becomes `http://oauth2:<token>@host/path`.
fn embed_token(url: &str, token: &str) -> Result<String> {
  if token.is_empty() {
    bail!("--with-token needs a stored personal access token");
  }
  let Some((scheme, rest)) = url.split_once("://") else {
    bail!("clone URL {url} does not use a scheme");
  };
  Ok(format!("{scheme}://oauth2:{token}@{rest}"))
}
