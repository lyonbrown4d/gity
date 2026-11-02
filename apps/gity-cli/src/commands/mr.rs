//! `gity mr` — list, view, create, merge and inspect merge requests.
use crate::client::{Client, Options};
use crate::format;
use anyhow::Result;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum MrCommand {
  /// List merge requests in a project
  List {
    project: String,
    /// Filter by state: opened, merged, closed, or all
    #[arg(long, default_value = "opened")]
    state: String,
    #[arg(long, default_value = "1")]
    page: u64,
    #[arg(long = "page-size", default_value = "20")]
    page_size: u64,
  },
  /// Show one merge request
  View { project: String, iid: i64 },
  /// Open a merge request
  Create {
    project: String,
    #[arg(long)]
    title: String,
    #[arg(long)]
    source: String,
    #[arg(long)]
    target: String,
    #[arg(long, default_value = "")]
    description: String,
  },
  /// Merge a merge request
  Merge {
    project: String,
    iid: i64,
    /// Squash the source commits into one commit
    #[arg(long)]
    squash: bool,
  },
  /// Print the merge request diff
  Diff { project: String, iid: i64 },
}

pub async fn run(command: MrCommand, options: &Options) -> Result<()> {
  let client = options.client()?;
  match command {
    MrCommand::List {
      project,
      state,
      page,
      page_size,
    } => list(&client, options, &project, &state, page, page_size).await,
    MrCommand::View { project, iid } => view(&client, options, &project, iid).await,
    MrCommand::Create {
      project,
      title,
      source,
      target,
      description,
    } => create(&client, &project, title, source, target, description).await,
    MrCommand::Merge {
      project,
      iid,
      squash,
    } => merge(&client, &project, iid, squash).await,
    MrCommand::Diff { project, iid } => diff(&client, &project, iid).await,
  }
}

async fn list(
  client: &Client,
  options: &Options,
  project: &str,
  state: &str,
  page: u64,
  page_size: u64,
) -> Result<()> {
  let id = client.project_id(project).await?;
  let merge_requests = client
    .get(&format!(
      "/projects/{id}/merge-requests?state={state}&page={page}&page_size={page_size}"
    ))
    .await?;
  if options.json {
    format::print_json(&merge_requests);
    return Ok(());
  }
  let rows = merge_requests["items"]
    .as_array()
    .into_iter()
    .flatten()
    .map(|mr| {
      vec![
        format::text(mr, "iid"),
        format::text(mr, "state"),
        format::ellipsis(format::field(mr, "title"), 44),
        format!(
          "{} → {}",
          format::text(mr, "source_branch"),
          format::text(mr, "target_branch")
        ),
      ]
    })
    .collect::<Vec<_>>();
  format::table(&["!", "STATE", "TITLE", "BRANCHES"], &rows);
  Ok(())
}

async fn view(client: &Client, options: &Options, project: &str, iid: i64) -> Result<()> {
  let id = client.project_id(project).await?;
  let mr = client
    .get(&format!("/projects/{id}/merge-requests/{iid}"))
    .await?;
  if options.json {
    format::print_json(&mr);
    return Ok(());
  }
  println!("!{iid} {}", format::text(&mr, "title"));
  println!("state: {}", format::text(&mr, "state"));
  println!(
    "branches: {} → {}",
    format::text(&mr, "source_branch"),
    format::text(&mr, "target_branch")
  );
  let description = format::text(&mr, "description");
  if !description.is_empty() {
    println!("\n{description}");
  }
  let checks = client
    .get(&format!("/projects/{id}/merge-requests/{iid}/checks"))
    .await?;
  if let Some(checks) = checks.as_array().filter(|checks| !checks.is_empty()) {
    println!("\nchecks:");
    for check in checks {
      println!(
        "  {} {}",
        format::text(check, "status"),
        format::text(check, "name")
      );
    }
  }
  Ok(())
}

async fn create(
  client: &Client,
  project: &str,
  title: String,
  source: String,
  target: String,
  description: String,
) -> Result<()> {
  let id = client.project_id(project).await?;
  let mut body = json!({
    "title": title,
    "source_branch": source,
    "target_branch": target,
  });
  if !description.is_empty() {
    body["description"] = json!(description);
  }
  let mr = client
    .post(&format!("/projects/{id}/merge-requests"), body)
    .await?;
  println!(
    "opened !{} {}",
    format::text(&mr, "iid"),
    format::text(&mr, "title")
  );
  Ok(())
}

async fn merge(client: &Client, project: &str, iid: i64, squash: bool) -> Result<()> {
  let id = client.project_id(project).await?;
  let result = client
    .post(
      &format!("/projects/{id}/merge-requests/{iid}/merge"),
      json!({ "squash": squash }),
    )
    .await?;
  println!("merged !{iid}: {}", format::text(&result, "state"));
  Ok(())
}

async fn diff(client: &Client, project: &str, iid: i64) -> Result<()> {
  let id = client.project_id(project).await?;
  let diff = client
    .get(&format!("/projects/{id}/merge-requests/{iid}/diff"))
    .await?;
  match diff.as_str() {
    Some(text) => println!("{text}"),
    None => format::print_json(&diff),
  }
  Ok(())
}
