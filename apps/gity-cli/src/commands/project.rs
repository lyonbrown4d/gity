//! `gity project` — list and inspect projects.
use crate::client::{Client, Options};
use crate::format;
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ProjectCommand {
  /// List projects visible to the authenticated user
  List,
  /// Show one project by id or `group/project` path
  View {
    /// Project id or path
    project: String,
  },
}

pub async fn run(command: ProjectCommand, options: &Options) -> Result<()> {
  let client = options.client()?;
  match command {
    ProjectCommand::List => list(&client, options).await,
    ProjectCommand::View { project } => view(&client, options, &project).await,
  }
}

async fn list(client: &Client, options: &Options) -> Result<()> {
  let projects = client.get("/projects").await?;
  if options.json {
    format::print_json(&projects);
    return Ok(());
  }
  let rows = projects
    .as_array()
    .into_iter()
    .flatten()
    .map(|project| {
      vec![
        format::text(project, "id"),
        format::text(project, "full_path"),
        format::text(project, "visibility"),
        format::ellipsis(format::field(project, "name"), 40),
      ]
    })
    .collect::<Vec<_>>();
  format::table(&["ID", "PATH", "VISIBILITY", "NAME"], &rows);
  Ok(())
}

async fn view(client: &Client, options: &Options, reference: &str) -> Result<()> {
  let id = client.project_id(reference).await?;
  let project = client.get(&format!("/projects/{id}")).await?;
  if options.json {
    format::print_json(&project);
    return Ok(());
  }
  for (label, key) in [
    ("id", "id"),
    ("path", "full_path"),
    ("name", "name"),
    ("visibility", "visibility"),
    ("default branch", "default_branch"),
    ("clone", "clone_http_url"),
  ] {
    let value = format::text(&project, key);
    if !value.is_empty() {
      println!("{label:>15}: {value}");
    }
  }
  Ok(())
}
