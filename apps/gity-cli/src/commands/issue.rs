//! `gity issue` — list, view, create, close, comment, assign, label, milestone.
use crate::client::{Client, Options};
use crate::format;
use anyhow::Result;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum IssueCommand {
  /// List issues in a project
  List {
    project: String,
    /// Filter by state: open, closed, or all
    #[arg(long, default_value = "open")]
    state: String,
    #[arg(long, default_value = "1")]
    page: u64,
    #[arg(long = "page-size", default_value = "20")]
    page_size: u64,
  },
  /// Show one issue by its project-local number
  View { project: String, iid: i64 },
  /// Create an issue
  Create {
    project: String,
    #[arg(long)]
    title: String,
    #[arg(long, default_value = "")]
    description: String,
  },
  /// Close an issue
  Close { project: String, iid: i64 },
  /// Add a comment to an issue
  Comment {
    project: String,
    iid: i64,
    #[arg(long)]
    body: String,
  },
  /// Replace the assignee set with the given user ids
  Assign {
    project: String,
    iid: i64,
    #[arg(required = true)]
    users: Vec<i64>,
  },
  /// Remove users from the assignee set
  Unassign {
    project: String,
    iid: i64,
    #[arg(required = true)]
    users: Vec<i64>,
  },
  /// Print the labels of one issue
  Labels { project: String, iid: i64 },
  /// Add or remove labels by name
  Label {
    project: String,
    iid: i64,
    #[arg(long)]
    add: Vec<String>,
    #[arg(long)]
    remove: Vec<String>,
  },
  /// Set or clear the milestone of one issue
  Milestone {
    project: String,
    iid: i64,
    #[arg(long)]
    set: Option<i64>,
    #[arg(long)]
    clear: bool,
  },
  /// List the milestones defined in a project
  Milestones { project: String },
}

pub async fn run(command: IssueCommand, options: &Options) -> Result<()> {
  let client = options.client()?;
  match command {
    IssueCommand::List {
      project,
      state,
      page,
      page_size,
    } => list(&client, options, &project, &state, page, page_size).await,
    IssueCommand::View { project, iid } => view(&client, options, &project, iid).await,
    IssueCommand::Create {
      project,
      title,
      description,
    } => create(&client, &project, title, description).await,
    IssueCommand::Close { project, iid } => close(&client, options, &project, iid).await,
    IssueCommand::Comment { project, iid, body } => comment(&client, &project, iid, body).await,
    IssueCommand::Assign {
      project,
      iid,
      users,
    } => assign(&client, &project, iid, users).await,
    IssueCommand::Unassign {
      project,
      iid,
      users,
    } => unassign(&client, &project, iid, users).await,
    IssueCommand::Labels { project, iid } => labels(&client, options, &project, iid).await,
    IssueCommand::Label {
      project,
      iid,
      add,
      remove,
    } => label(&client, &project, iid, add, remove).await,
    IssueCommand::Milestone {
      project,
      iid,
      set,
      clear,
    } => milestone(&client, &project, iid, set, clear).await,
    IssueCommand::Milestones { project } => milestones(&client, options, &project).await,
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
  let issues = client
    .get(&format!(
      "/projects/{id}/issues?state={state}&page={page}&page_size={page_size}"
    ))
    .await?;
  if options.json {
    format::print_json(&issues);
    return Ok(());
  }
  let rows = issues["items"]
    .as_array()
    .into_iter()
    .flatten()
    .map(|issue| {
      vec![
        format::text(issue, "number"),
        format::text(issue, "status"),
        format::ellipsis(format::field(issue, "title"), 48),
        format::text(issue, "author_user_id"),
      ]
    })
    .collect::<Vec<_>>();
  format::table(&["#", "STATE", "TITLE", "AUTHOR"], &rows);
  Ok(())
}

async fn view(client: &Client, options: &Options, project: &str, iid: i64) -> Result<()> {
  let id = client.project_id(project).await?;
  let issue = client.get(&format!("/projects/{id}/issues/{iid}")).await?;
  if options.json {
    format::print_json(&issue);
    return Ok(());
  }
  println!(
    "#{} {}",
    format::text(&issue, "number"),
    format::text(&issue, "title")
  );
  println!("state: {}", format::text(&issue, "status"));
  let labels = names(&issue, "labels");
  if !labels.is_empty() {
    println!("labels: {labels}");
  }
  let assignees = names(&issue, "assignees");
  if !assignees.is_empty() {
    println!("assignees: {assignees}");
  }
  let milestone = format::text(&issue, "milestone_id");
  if !milestone.is_empty() {
    println!("milestone: {milestone}");
  }
  let description = format::text(&issue, "description");
  if !description.is_empty() {
    println!("\n{description}");
  }
  for comment in client
    .get(&format!("/projects/{id}/issues/{iid}/comments"))
    .await?
    .as_array()
    .into_iter()
    .flatten()
  {
    println!(
      "\n— {}:\n{}",
      format::text(comment, "author_user_id"),
      format::text(comment, "content")
    );
  }
  Ok(())
}

async fn create(client: &Client, project: &str, title: String, description: String) -> Result<()> {
  let id = client.project_id(project).await?;
  let mut body = json!({ "title": title });
  if !description.is_empty() {
    body["description"] = json!(description);
  }
  let issue = client.post(&format!("/projects/{id}/issues"), body).await?;
  println!(
    "created #{} {}",
    format::text(&issue, "number"),
    format::text(&issue, "title")
  );
  Ok(())
}

async fn close(client: &Client, options: &Options, project: &str, iid: i64) -> Result<()> {
  let id = client.project_id(project).await?;
  let issue = client
    .patch(
      &format!("/projects/{id}/issues/{iid}"),
      json!({ "state": "closed" }),
    )
    .await?;
  if options.json {
    format::print_json(&issue);
  } else {
    println!("closed #{}", format::text(&issue, "number"));
  }
  Ok(())
}

use super::issue_collab::{assign, comment, label, labels, milestone, milestones, names, unassign};
