//! `gity issue` — list, view, create, close and comment on issues.
use crate::client::{self, Client, Options};
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
}

pub async fn run(command: IssueCommand, options: &Options) -> Result<()> {
  let client = Client::from_options(options)?;
  match command {
    IssueCommand::List { project, state } => list(&client, options, &project, &state).await,
    IssueCommand::View { project, iid } => view(&client, options, &project, iid).await,
    IssueCommand::Create {
      project,
      title,
      description,
    } => create(&client, &project, title, description).await,
    IssueCommand::Close { project, iid } => close(&client, options, &project, iid).await,
    IssueCommand::Comment { project, iid, body } => comment(&client, &project, iid, body).await,
  }
}

async fn list(client: &Client, options: &Options, project: &str, state: &str) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let path = match state {
    "all" => format!("/projects/{id}/issues"),
    other => format!("/projects/{id}/issues?state={other}"),
  };
  let issues = client.get(&path).await?;
  if options.json {
    format::print_json(&issues);
    return Ok(());
  }
  let rows = issues
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
  let id = client::project_id(client, project).await?;
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
  let description = format::text(&issue, "description");
  if !description.is_empty() {
    println!("\n{description}");
  }
  let comments = client
    .get(&format!("/projects/{id}/issues/{iid}/comments"))
    .await?;
  for comment in comments.as_array().into_iter().flatten() {
    println!(
      "\n— {}:\n{}",
      format::text(comment, "author_user_id"),
      format::text(comment, "content")
    );
  }
  Ok(())
}

async fn create(client: &Client, project: &str, title: String, description: String) -> Result<()> {
  let id = client::project_id(client, project).await?;
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
  let id = client::project_id(client, project).await?;
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

async fn comment(client: &Client, project: &str, iid: i64, body: String) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let comment = client
    .post(
      &format!("/projects/{id}/issues/{iid}/comments"),
      json!({ "body": body }),
    )
    .await?;
  println!("comment {} added", format::text(&comment, "id"));
  Ok(())
}
