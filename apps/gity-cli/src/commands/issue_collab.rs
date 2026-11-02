//! Collaborative `gity issue` subcommands: comments, assignees, labels, milestones.
use crate::client::{Client, Options};
use crate::format;
use anyhow::{Result, bail};
use serde_json::{Value, json};

pub(super) async fn comment(client: &Client, project: &str, iid: i64, body: String) -> Result<()> {
  let id = client.project_id(project).await?;
  let comment = client
    .post(
      &format!("/projects/{id}/issues/{iid}/comments"),
      json!({ "body": body }),
    )
    .await?;
  println!("comment {} added", format::text(&comment, "id"));
  Ok(())
}

pub(super) async fn assign(
  client: &Client,
  project: &str,
  iid: i64,
  users: Vec<i64>,
) -> Result<()> {
  let id = client.project_id(project).await?;
  let path = format!("/projects/{id}/issues/{iid}/assignees");
  client.patch(&path, json!({ "user_ids": users })).await?;
  let assignees = client.get(&path).await?;
  println!("assignees: {}", names(&assignees, "assignees"));
  Ok(())
}

pub(super) async fn unassign(
  client: &Client,
  project: &str,
  iid: i64,
  users: Vec<i64>,
) -> Result<()> {
  let id = client.project_id(project).await?;
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

pub(super) async fn labels(
  client: &Client,
  options: &Options,
  project: &str,
  iid: i64,
) -> Result<()> {
  let id = client.project_id(project).await?;
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
pub(super) async fn label(
  client: &Client,
  project: &str,
  iid: i64,
  add: Vec<String>,
  remove: Vec<String>,
) -> Result<()> {
  if add.is_empty() && remove.is_empty() {
    bail!("pass --add and/or --remove");
  }
  let id = client.project_id(project).await?;
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

pub(super) async fn milestone(
  client: &Client,
  project: &str,
  iid: i64,
  set: Option<i64>,
  clear: bool,
) -> Result<()> {
  let id = client.project_id(project).await?;
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

pub(super) async fn milestones(client: &Client, options: &Options, project: &str) -> Result<()> {
  let id = client.project_id(project).await?;
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
pub(super) async fn resolve_milestone(
  client: &Client,
  project: &str,
  reference: i64,
) -> Result<i64> {
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
pub(super) fn label_name(label: &Value) -> String {
  ["name", "title"]
    .iter()
    .find_map(|field| label[*field].as_str().map(str::to_owned))
    .unwrap_or_default()
}

pub(super) fn names(value: &Value, key: &str) -> String {
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
