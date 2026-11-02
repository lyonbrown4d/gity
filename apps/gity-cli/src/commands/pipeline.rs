//! `gity pipeline` — list, inspect, run, retry and cancel pipelines.
use crate::client::{self, Client, Options};
use crate::format;
use anyhow::Result;
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand)]
pub enum PipelineCommand {
  /// List pipelines for a project
  List { project: String },
  /// Show one pipeline and its jobs
  View { project: String, pipeline: i64 },
  /// List the jobs of one pipeline
  Jobs { project: String, pipeline: i64 },
  /// Run a pipeline against a ref; the server resolves the commit and config
  Run {
    project: String,
    /// Branch or tag to run; defaults to the project default branch
    #[arg(long = "ref")]
    ref_name: Option<String>,
    /// Explicit commit SHA, skipping ref resolution
    #[arg(long)]
    sha: Option<String>,
  },
  /// Retry a finished pipeline with new job attempts
  Retry { project: String, pipeline: i64 },
  /// Cancel a running pipeline
  Cancel { project: String, pipeline: i64 },
}

pub async fn run(command: PipelineCommand, options: &Options) -> Result<()> {
  let client = Client::from_options(options)?;
  match command {
    PipelineCommand::List { project } => list(&client, options, &project).await,
    PipelineCommand::View { project, pipeline } => view(&client, options, &project, pipeline).await,
    PipelineCommand::Jobs { project, pipeline } => jobs(&client, options, &project, pipeline).await,
    PipelineCommand::Run {
      project,
      ref_name,
      sha,
    } => run_pipeline(&client, options, &project, ref_name, sha).await,
    PipelineCommand::Retry { project, pipeline } => {
      act(&client, options, &project, pipeline, "retry").await
    }
    PipelineCommand::Cancel { project, pipeline } => {
      act(&client, options, &project, pipeline, "cancel").await
    }
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
        format::text(pipeline, "iid"),
        format::ellipsis(format::field(pipeline, "name"), 24),
        format::text(pipeline, "status"),
        format::text(pipeline, "ref_name"),
        format::ellipsis(format::field(pipeline, "commit_sha"), 12),
      ]
    })
    .collect::<Vec<_>>();
  format::table(&["ID", "IID", "NAME", "STATUS", "REF", "SHA"], &rows);
  Ok(())
}

async fn view(client: &Client, options: &Options, project: &str, pipeline: i64) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let detail = client
    .get(&format!("/projects/{id}/pipelines/{pipeline}"))
    .await?;
  if options.json {
    format::print_json(&detail);
    return Ok(());
  }
  let body = pipeline_of(&detail);
  for key in [
    "id",
    "iid",
    "name",
    "status",
    "ref_name",
    "commit_sha",
    "created_at",
  ] {
    let value = format::text(body, key);
    if !value.is_empty() {
      println!("{key:>12}: {value}");
    }
  }
  let jobs = detail["jobs"].as_array().into_iter().flatten();
  if jobs.clone().count() > 0 {
    println!();
    print_jobs(jobs);
  }
  Ok(())
}

async fn jobs(client: &Client, options: &Options, project: &str, pipeline: i64) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let jobs = client
    .get(&format!("/projects/{id}/pipelines/{pipeline}/jobs"))
    .await?;
  if options.json {
    format::print_json(&jobs);
    return Ok(());
  }
  print_jobs(jobs.as_array().into_iter().flatten());
  Ok(())
}

async fn run_pipeline(
  client: &Client,
  options: &Options,
  project: &str,
  ref_name: Option<String>,
  sha: Option<String>,
) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let mut body = json!({ "source": "cli" });
  if let Some(ref_name) = ref_name {
    body["ref_name"] = json!(ref_name);
  }
  if let Some(sha) = sha {
    body["commit_sha"] = json!(sha);
  }
  let detail = client
    .post(&format!("/projects/{id}/pipelines"), body)
    .await?;
  if options.json {
    format::print_json(&detail);
    return Ok(());
  }
  let pipeline = pipeline_of(&detail);
  println!(
    "started pipeline {} (#{}) on {} at {}",
    format::text(pipeline, "id"),
    format::text(pipeline, "iid"),
    format::text(pipeline, "ref_name"),
    format::ellipsis(format::field(pipeline, "commit_sha"), 12)
  );
  Ok(())
}

async fn act(
  client: &Client,
  options: &Options,
  project: &str,
  pipeline: i64,
  action: &str,
) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let result = client
    .post(
      &format!("/projects/{id}/pipelines/{pipeline}/{action}"),
      json!({}),
    )
    .await?;
  if options.json {
    format::print_json(&result);
  } else {
    println!(
      "{action} pipeline {pipeline}: {}",
      format::text(pipeline_of(&result), "status")
    );
  }
  Ok(())
}

/// Pipeline responses wrap the record as `{pipeline, jobs}`; list rows are bare.
fn pipeline_of(value: &Value) -> &Value {
  if value.get("pipeline").is_some_and(Value::is_object) {
    &value["pipeline"]
  } else {
    value
  }
}

fn print_jobs<'a>(jobs: impl Iterator<Item = &'a Value>) {
  let rows = jobs
    .map(|job| {
      let pipeline_job = &job["pipeline_job"];
      let nested = pipeline_job.is_object();
      vec![
        format::text(if nested { pipeline_job } else { job }, "id"),
        format::text(if nested { pipeline_job } else { job }, "name"),
        format::text(job, "status"),
        format::text(if nested { pipeline_job } else { job }, "stage"),
      ]
    })
    .collect::<Vec<_>>();
  format::table(&["ID", "NAME", "STATUS", "STAGE"], &rows);
}
