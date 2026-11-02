//! `gity issue` — list, view, create, close, comment, assign, label, milestone.
use crate::client::{self, Client, Options};
use crate::format;
use anyhow::{Result, bail};
use clap::Subcommand;
use serde_json::{Value, json};

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

async fn list(client: &Client, options: &Options, project: &str, state: &str) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let issues = client.get(&format!("/projects/{id}/issues")).await?;
  let issues = Value::Array(format::filter_state(&issues, "status", state));
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

async fn assign(client: &Client, project: &str, iid: i64, users: Vec<i64>) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let path = format!("/projects/{id}/issues/{iid}/assignees");
  client.patch(&path, json!({ "user_ids": users })).await?;
  let assignees = client.get(&path).await?;
  println!("assignees: {}", names(&assignees, "assignees"));
  Ok(())
}

async fn unassign(client: &Client, project: &str, iid: i64, users: Vec<i64>) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let path = format!("/projects/{id}/issues/{iid}/assignees");
  let current = client.get(&path).await?;
  let keep = current
    .as_array()
    .into_iter()
    .flatten()
    .filter_map(|row| row["user_id"].as_i64())
    .filter(|user| !users.contains(user))
    .collect::<Vec<_>>();
  client.patch(&path, json!({ "user_ids": keep })).await?;
  let remaining = client.get(&path).await?;
  println!("assignees: {}", names(&remaining, "assignees"));
  Ok(())
}

async fn labels(client: &Client, options: &Options, project: &str, iid: i64) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let labels = client
    .get(&format!("/projects/{id}/issues/{iid}/labels"))
    .await?;
  if options.json {
    format::print_json(&labels);
    return Ok(());
  }
  let rows = labels
    .as_array()
    .into_iter()
    .flatten()
    .map(|label| vec![label_name(label)])
    .collect::<Vec<_>>();
  format::table(&["LABEL"], &rows);
  Ok(())
}

/// The endpoint replaces the whole label set, so compute it from the current one.
async fn label(
  client: &Client,
  project: &str,
  iid: i64,
  add: Vec<String>,
  remove: Vec<String>,
) -> Result<()> {
  if add.is_empty() && remove.is_empty() {
    bail!("pass --add and/or --remove");
  }
  let id = client::project_id(client, project).await?;
  let path = format!("/projects/{id}/issues/{iid}/labels");
  let current = client.get(&path).await?;
  let mut labels = current
    .as_array()
    .into_iter()
    .flatten()
    .map(label_name)
    .collect::<Vec<_>>();
  labels.retain(|label| !remove.contains(label));
  for label in add {
    if !labels.contains(&label) {
      labels.push(label);
    }
  }
  client
    .patch(&path, json!({ "names": labels.clone() }))
    .await?;
  let updated = client.get(&path).await?;
  let applied = updated
    .as_array()
    .into_iter()
    .flatten()
    .map(label_name)
    .collect::<Vec<_>>();
  println!("labels: {}", applied.join(", "));
  let missing = labels
    .iter()
    .filter(|label| !applied.contains(label))
    .cloned()
    .collect::<Vec<_>>();
  if !missing.is_empty() {
    println!(
      "not applied (create them with `gity api /projects/<id>/labels`): {}",
      missing.join(", ")
    );
  }
  Ok(())
}

async fn milestone(
  client: &Client,
  project: &str,
  iid: i64,
  set: Option<i64>,
  clear: bool,
) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let value = match (set, clear) {
    (Some(milestone), false) => json!(resolve_milestone(client, &id, milestone).await?),
    (None, true) => Value::Null,
    _ => bail!("pass either --set <milestone> or --clear"),
  };
  client
    .patch(
      &format!("/projects/{id}/issues/{iid}"),
      json!({ "milestone_id": value }),
    )
    .await?;
  println!("milestone updated for #{iid}");
  Ok(())
}

async fn milestones(client: &Client, options: &Options, project: &str) -> Result<()> {
  let id = client::project_id(client, project).await?;
  let milestones = client.get(&format!("/projects/{id}/milestones")).await?;
  if options.json {
    format::print_json(&milestones);
    return Ok(());
  }
  let rows = milestones
    .as_array()
    .into_iter()
    .flatten()
    .map(|milestone| {
      vec![
        format::text(milestone, "iid"),
        format::text(milestone, "title"),
        format::text(milestone, "state"),
        format::text(milestone, "due_date"),
      ]
    })
    .collect::<Vec<_>>();
  format::table(&["IID", "TITLE", "STATE", "DUE"], &rows);
  Ok(())
}

/// `--set` takes the project-local milestone number; the API stores the entity id.
async fn resolve_milestone(client: &Client, project: &str, reference: i64) -> Result<i64> {
  let milestones = client
    .get(&format!("/projects/{project}/milestones"))
    .await?;
  let found = milestones
    .as_array()
    .into_iter()
    .flatten()
    .find(|row| {
      row["iid"].as_i64() == Some(reference) || row["id"].as_str() == Some(&reference.to_string())
    })
    .ok_or_else(|| {
      anyhow::anyhow!("milestone {reference} not found; run `gity issue milestones <project>`")
    })?;
  found["id"]
    .as_str()
    .and_then(|id| id.parse::<i64>().ok())
    .or_else(|| found["id"].as_i64())
    .ok_or_else(|| anyhow::anyhow!("milestone {reference} has no usable id"))
}

/// Label payloads use `name`; issue payloads may carry either shape.
fn label_name(label: &Value) -> String {
  ["name", "title"]
    .iter()
    .find_map(|field| label[*field].as_str().map(str::to_owned))
    .unwrap_or_default()
}

fn names(value: &Value, key: &str) -> String {
  // Collection endpoints return a bare array; issue payloads nest it under a key.
  value[key]
    .as_array()
    .or_else(|| value.as_array())
    .into_iter()
    .flatten()
    .map(|row| {
      ["name", "title", "username", "user_id", "id"]
        .iter()
        .find_map(|field| match &row[*field] {
          Value::String(text) if !text.is_empty() => Some(text.clone()),
          Value::Number(number) => Some(number.to_string()),
          _ => None,
        })
        .unwrap_or_default()
    })
    .collect::<Vec<_>>()
    .join(", ")
}
