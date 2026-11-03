use super::{schemas::*, scope::Scope};
use crate::AppState;
use gity_core::AppError;
use gity_search::global::SearchDocument;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
pub type IssueRows = HashMap<(i64, String, i64), gity_entities::project_issues::Model>;

pub async fn hit(
  s: &AppState,
  scope: &Scope,
  doc: &SearchDocument,
  query: &str,
  kind: SearchKind,
  issues: &IssueRows,
) -> Result<Option<SearchHit>, AppError> {
  let Some(project) = scope
    .projects
    .iter()
    .find(|p| p.project.id == doc.project_id)
  else {
    return Ok(None);
  };
  let mut hit = SearchHit {
    kind,
    project_id: doc.project_id.to_string(),
    organization_id: project.project.organization_id.to_string(),
    project_path: project.project.full_path.clone(),
    resource_id: doc.resource_id.clone(),
    title: String::new(),
    snippet: String::new(),
    path: None,
    commit_sha: None,
    line: None,
    issue_iid: None,
    state: None,
  };
  match kind {
    SearchKind::Issue => {
      let Some(id) = doc.resource_id.parse::<i64>().ok() else {
        return Ok(None);
      };
      let Some(row) = issues.get(&(doc.project_id, "issue".into(), id)) else {
        return Ok(None);
      };
      if row.updated_at.to_string() != doc.version {
        return Ok(None);
      }
      let assigned = scope.confidential_ids.contains(&row.id);
      if !crate::features::issue_access::visible(
        &row,
        Some(scope.actor.user_id),
        project.level,
        assigned,
        false,
      ) {
        return Ok(None);
      }
      let body = row.description.as_deref().unwrap_or_default();
      if !matches(query, &row.title) && !matches(query, body) {
        return Ok(None);
      }
      hit.title = row.title.clone();
      hit.snippet = snippet(query, body);
      hit.issue_iid = Some(row.iid.to_string());
      hit.state = Some(row.state.clone());
    }
    SearchKind::Code => {
      let repo = format!("{}.git", doc.project_id);
      let head = s
        .git
        .resolve_revision(&repo, &project.project.default_branch, "")
        .await
        .map_err(|_| {
          AppError::new(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "Git search source unavailable",
          )
        })?;
      if head != doc.version {
        return Ok(None);
      }
      let blob = s
        .git
        .get_blob_limited(
          &repo,
          &head,
          "",
          &doc.path,
          s.settings.search.max_file_size_bytes,
        )
        .await
        .map_err(|_| {
          AppError::new(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "Git search source unavailable",
          )
        })?;
      if blob["encoding"] != "utf-8" {
        return Ok(None);
      }
      let body = blob["content"].as_str().unwrap_or_default();
      if hex::encode(Sha256::digest(body.as_bytes())) != doc.resource_id {
        return Ok(None);
      }
      if !matches(query, &doc.path) && !matches(query, body) {
        return Ok(None);
      }
      let lower = body.to_lowercase();
      let line = lower
        .find(&query.to_lowercase())
        .map(|byte| lower[..byte].bytes().filter(|b| *b == b'\n').count() + 1)
        .unwrap_or(1);
      hit.title = doc.path.clone();
      hit.path = Some(doc.path.clone());
      hit.commit_sha = Some(head);
      hit.line = Some(line as u32);
      hit.snippet = snippet(query, body);
    }
  }
  Ok(Some(hit))
}
fn matches(query: &str, value: &str) -> bool {
  value.to_lowercase().contains(&query.to_lowercase())
}
fn snippet(query: &str, value: &str) -> String {
  let line = value
    .lines()
    .find(|line| matches(query, line))
    .unwrap_or_else(|| value.lines().next().unwrap_or(""));
  let lower = line.to_lowercase();
  let start = lower
    .find(&query.to_lowercase())
    .map(|byte| lower[..byte].chars().count().saturating_sub(120))
    .unwrap_or(0);
  line.chars().skip(start).take(320).collect()
}

/// Recheck the exact returned source rows after hydration, including a concurrent
/// confidentiality change. The permission scope is resolved again by the caller.
pub async fn revalidate(
  s: &AppState,
  scope: &Scope,
  groups: &mut [SearchGroup],
  before: &IssueRows,
) -> Result<(), AppError> {
  let targets = groups
    .iter()
    .flat_map(|g| &g.items)
    .filter(|hit| hit.kind == SearchKind::Issue)
    .filter_map(|hit| {
      Some((
        hit.project_id.parse().ok()?,
        "issue".to_owned(),
        hit.resource_id.parse().ok()?,
      ))
    })
    .collect::<Vec<_>>();
  let current = s.collaboration.issue_targets(&targets).await?;
  for group in groups {
    group.items.retain(|hit| {
      if hit.kind != SearchKind::Issue {
        return true;
      }
      let (Ok(project), Ok(id)) = (
        hit.project_id.parse::<i64>(),
        hit.resource_id.parse::<i64>(),
      ) else {
        return false;
      };
      let key = (project, "issue".to_owned(), id);
      let (Some(row), Some(old)) = (current.get(&key), before.get(&key)) else {
        return false;
      };
      let Some(project) = scope.projects.iter().find(|p| p.project.id == project) else {
        return false;
      };
      old.updated_at == row.updated_at
        && crate::features::issue_access::visible(
          row,
          Some(scope.actor.user_id),
          project.level,
          scope.confidential_ids.contains(&id),
          false,
        )
    });
  }
  Ok(())
}
