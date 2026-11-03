//! Recheck project and confidential issue visibility when returning saved feeds.
use super::*;
use axum::{http::HeaderMap, http::StatusCode};
use gity_collaboration::activity::ActivityItem;
use gity_entities::{project_issues, project_subscriptions, project_todos};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct Target {
  project: i64,
  target_type: String,
  target_id: String,
}

pub(super) async fn filter_todos(
  state: &AppState,
  headers: &HeaderMap,
  rows: Vec<project_todos::Model>,
) -> Result<Vec<project_todos::Model>, AppError> {
  let targets = rows
    .iter()
    .map(|row| Target {
      project: row.project_id,
      target_type: row.target_type.clone(),
      target_id: row.target_id.clone(),
    })
    .collect::<Vec<_>>();
  let allowed = visible_targets(state, headers, &targets).await?;
  Ok(
    rows
      .into_iter()
      .filter(|row| {
        allowed.contains(&(
          row.project_id,
          row.target_type.clone(),
          row.target_id.clone(),
        ))
      })
      .collect(),
  )
}

pub(super) async fn filter_activity(
  state: &AppState,
  headers: &HeaderMap,
  rows: Vec<ActivityItem>,
) -> Result<Vec<ActivityItem>, AppError> {
  let targets = rows
    .iter()
    .map(|row| Target {
      project: row.event.project_id,
      target_type: row.event.target_type.clone(),
      target_id: row.event.target_id.clone(),
    })
    .collect::<Vec<_>>();
  let allowed = visible_targets(state, headers, &targets).await?;
  Ok(
    rows
      .into_iter()
      .filter(|row| {
        allowed.contains(&(
          row.event.project_id,
          row.event.target_type.clone(),
          row.event.target_id.clone(),
        ))
      })
      .collect(),
  )
}

pub(super) async fn filter_subscriptions(
  state: &AppState,
  headers: &HeaderMap,
  rows: Vec<project_subscriptions::Model>,
) -> Result<Vec<project_subscriptions::Model>, AppError> {
  let targets = rows
    .iter()
    .map(|row| Target {
      project: row.project_id,
      target_type: row.target_type.clone(),
      target_id: row.target_id.to_string(),
    })
    .collect::<Vec<_>>();
  let allowed = visible_targets(state, headers, &targets).await?;
  Ok(
    rows
      .into_iter()
      .filter(|row| {
        allowed.contains(&(
          row.project_id,
          row.target_type.clone(),
          row.target_id.to_string(),
        ))
      })
      .collect(),
  )
}

async fn visible_targets(
  state: &AppState,
  headers: &HeaderMap,
  targets: &[Target],
) -> Result<HashSet<(i64, String, String)>, AppError> {
  let projects = targets
    .iter()
    .map(|target| target.project)
    .collect::<HashSet<_>>();
  let mut authorizations = HashMap::new();
  for project in projects {
    match crate::access::project_action(state, headers, project, ProjectAction::Read).await {
      Ok(authorization) => {
        authorizations.insert(project, authorization);
      }
      Err(error)
        if error.status == StatusCode::FORBIDDEN || error.status == StatusCode::NOT_FOUND =>
      {
        // A formerly readable project is now hidden from this saved feed.
      }
      Err(error) => return Err(error),
    }
  }

  let issue_targets = targets
    .iter()
    .filter(|target| is_issue_target(&target.target_type))
    .filter_map(|target| {
      target
        .target_id
        .parse::<i64>()
        .ok()
        .map(|id| (target.project, target.target_type.clone(), id))
    })
    .collect::<Vec<_>>();
  let resolved = state.collaboration.issue_targets(&issue_targets).await?;
  let mut models_by_project: HashMap<i64, HashMap<i64, project_issues::Model>> = HashMap::new();
  for ((project, _, _), issue) in &resolved {
    if authorizations.contains_key(project) {
      models_by_project
        .entry(*project)
        .or_default()
        .insert(issue.id, issue.clone());
    }
  }

  let mut visible_issue_ids = HashMap::new();
  for (project, mut issues) in models_by_project {
    let authorization = authorizations
      .get(&project)
      .expect("issues were grouped only for authorized projects");
    let visible = super::issue_access::filter(
      state,
      authorization,
      issues.drain().map(|(_, issue)| issue).collect(),
    )
    .await?;
    visible_issue_ids.insert(
      project,
      visible
        .into_iter()
        .map(|issue| issue.id)
        .collect::<HashSet<_>>(),
    );
  }

  let mut allowed = HashSet::new();
  for target in targets {
    if !authorizations.contains_key(&target.project) {
      continue;
    }
    if !is_issue_target(&target.target_type) {
      allowed.insert((
        target.project,
        target.target_type.clone(),
        target.target_id.clone(),
      ));
      continue;
    }
    let Ok(target_id) = target.target_id.parse::<i64>() else {
      continue;
    };
    let key = (target.project, target.target_type.clone(), target_id);
    let Some(issue) = resolved.get(&key) else {
      continue;
    };
    if visible_issue_ids
      .get(&target.project)
      .is_some_and(|ids| ids.contains(&issue.id))
    {
      allowed.insert((
        target.project,
        target.target_type.clone(),
        target.target_id.clone(),
      ));
    }
  }
  Ok(allowed)
}

fn is_issue_target(target_type: &str) -> bool {
  target_type == "issue" || target_type.starts_with("issue_")
}
