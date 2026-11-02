//! `gity pipeline` — list pipelines, inspect one, and read its jobs.
use crate::client::{self, Client, Options};
use crate::format;
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum PipelineCommand {
  /// List pipelines for a project
  List { project: String },
  /// Show one pipeline
  View { project: String, pipeline: String },
  /// List the jobs of one pipeline
  Jobs { project: String, pipeline: String },
}

pub async fn run(command: PipelineCommand, options: &Options) -> Result<()> {
  let client = Client::from_options(options)?;
  match command {
    PipelineCommand::List { project } => list(&client, options, &project).await,
    PipelineCommand::View { project, pipeline } => {
      view(&client, options, &project, &pipeline).await
    }
    PipelineCommand::Jobs { project, pipeline } => jobs(&client, &project, &pipeline).await,
  }
}

async fn list(client: &Client, options: &Options, project: &str) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let pipelines = client.get(&format!("/projects/{id}/pipelines")).await?;
  if options.json {
    format::print_json(&pipelines);
    return Ok(());
  }
  let rows = pipelines
    .as_array()
    .into_iter()
    .flatten()
    .map(|pipeline| {
      vec![
        format::text(pipeline, "id"),
        status(pipeline),
        format::text(pipeline, "ref"),
        format::ellipsis(format::field(pipeline, "sha"), 12),
      ]
    })
    .collect::<Vec<_>>();
  format::table(&["ID", "STATUS", "REF", "SHA"], &rows);
  Ok(())
}

async fn view(client: &Client, options: &Options, project: &str, pipeline: &str) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let body = client
    .get(&format!("/projects/{id}/pipelines/{pipeline}"))
    .await?;
  if options.json {
    format::print_json(&body);
    return Ok(());
  }
  for key in ["id", "status", "state", "ref", "sha", "created_at"] {
    let value = format::text(&body, key);
    if !value.is_empty() {
      println!("{key:>12}: {value}");
    }
  }
  Ok(())
}

async fn jobs(client: &Client, project: &str, pipeline: &str) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let jobs = client
    .get(&format!("/projects/{id}/pipelines/{pipeline}/jobs"))
    .await?;
  let rows = jobs
    .as_array()
    .into_iter()
    .flatten()
    .map(|job| {
      vec![
        format::text(job, "id"),
        format::text(job, "name"),
        status(job),
        format::text(job, "stage"),
      ]
    })
    .collect::<Vec<_>>();
  format::table(&["ID", "NAME", "STATUS", "STAGE"], &rows);
  Ok(())
}

/// Pipelines and jobs expose their state under slightly different keys.
fn status(value: &serde_json::Value) -> String {
  let status = format::text(value, "status");
  if status.is_empty() {
    format::text(value, "state")
  } else {
    status
  }
}
