//! Resource authorization shared by REST and protocol-neutral operation adapters.
use super::*;
use crate::access::{ProjectAction, ProjectAuthorization};
use axum::http::HeaderMap;
use gity_entities::project_issues;

pub(crate) fn visible(
  issue: &project_issues::Model,
  user: Option<i64>,
  role: i64,
  assigned: bool,
  project_token: bool,
) -> bool {
  issue.confidential == 0
    || (!project_token
      && user.is_some_and(|user| role >= 20 || issue.author_user_id == user || assigned))
}

pub(crate) async fn authorize(
  s: &AppState,
  h: &HeaderMap,
  p: i64,
  i: i64,
  action: ProjectAction,
) -> Result<ProjectAuthorization, AppError> {
  let access = crate::access::project_action(s, h, p, action).await?;
  let row = s.collaboration.issue(p, i).await?;
  if !can_read(s, &access, &row).await? {
    return Err(AppError::not_found("issue not found"));
  }
  Ok(access)
}

pub(crate) async fn can_read(
  s: &AppState,
  access: &ProjectAuthorization,
  row: &project_issues::Model,
) -> Result<bool, AppError> {
  let user = access.actor().ok().map(|actor| actor.user_id);
  let assigned = if row.confidential != 0 && user.is_some() && access.role_level() < 20 {
    s.collaboration
      .issue_assignees(row.project_id, row.iid)
      .await?
      .iter()
      .any(|v| Some(v.user_id) == user)
  } else {
    false
  };
  Ok(visible(
    row,
    user,
    access.role_level(),
    assigned,
    access.is_project_token(),
  ))
}

pub(crate) async fn filter(
  s: &AppState,
  access: &ProjectAuthorization,
  rows: Vec<project_issues::Model>,
) -> Result<Vec<project_issues::Model>, AppError> {
  let user = access.actor().ok().map(|actor| actor.user_id);
  let ids = rows
    .iter()
    .filter(|v| v.confidential != 0)
    .map(|v| v.id)
    .collect::<Vec<_>>();
  let assigned = if access.role_level() < 20 && user.is_some() && !ids.is_empty() {
    s.collaboration
      .assigned_issues(number(&access.project, "id"), ids)
      .await?
      .into_iter()
      .filter(|v| Some(v.user_id) == user)
      .map(|v| v.project_issue_id)
      .collect::<std::collections::HashSet<_>>()
  } else {
    Default::default()
  };
  Ok(
    rows
      .into_iter()
      .filter(|v| {
        visible(
          v,
          user,
          access.role_level(),
          assigned.contains(&v.id),
          access.is_project_token(),
        )
      })
      .collect(),
  )
}
