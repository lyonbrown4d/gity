use super::{
  schemas::*,
  scope,
  session::{Group, Session},
};
use crate::AppState;
use axum::http::{HeaderMap, StatusCode};
use gity_core::AppError;
use gity_search::global::EngineQuery;
use sha2::{Digest, Sha256};
use std::{
  collections::HashSet,
  time::{Duration, Instant},
};
const CANDIDATE_BUDGET: usize = 1000;
fn unavailable() -> AppError {
  AppError::new(
    StatusCode::SERVICE_UNAVAILABLE,
    "search temporarily unavailable",
  )
}
pub async fn search(
  s: &AppState,
  h: &HeaderMap,
  mut input: SearchRequest,
) -> Result<SearchReply, AppError> {
  super::runtime::enabled(s)?;
  input.query = input.query.trim().to_owned();
  if input.query.is_empty()
    || input.query.len() > 512
    || input.limit == 0
    || input.limit > 50
    || input.types.is_empty()
    || input.types.len() > 2
  {
    return Err(AppError::bad_request("invalid search query or limit"));
  }
  let mut kinds = HashSet::new();
  for kind in &input.types {
    if !kinds.insert(format!("{kind:?}")) {
      return Err(AppError::bad_request("duplicate search type"));
    }
  }
  let scope = scope::resolve(s, h, &input).await?;
  let runtime = &s.global_search;
  let _slot = runtime
    .query_slots
    .try_acquire()
    .map_err(|_| unavailable())?;
  let cursor = input.cursor.take();
  let request = hex::encode(Sha256::digest(
    serde_json::to_vec(&input).map_err(|_| AppError::bad_request("invalid search request"))?,
  ));
  let mut session = if let Some(cursor) = cursor {
    let sessions = runtime.sessions.lock().map_err(|_| unavailable())?;
    let session = sessions
      .get(&cursor)
      .filter(|v| v.user == scope.actor.user_id && v.expires > Instant::now())
      .cloned()
      .ok_or_else(|| AppError::conflict("search cursor expired; restart search"))?;
    if session.scope != scope.fingerprint || session.request != request {
      return Err(AppError::conflict("search scope changed; restart search"));
    }
    session
  } else {
    let projects = scope
      .projects
      .iter()
      .map(|p| p.project.id)
      .collect::<Vec<_>>();
    let published = runtime
      .jobs
      .published(&projects)
      .await?
      .into_iter()
      .filter(|(_, token)| token.starts_with(&runtime.namespace_prefix))
      .collect::<std::collections::HashMap<_, _>>();
    let mut groups = Vec::new();
    let mut partial =
      published.len() != projects.len() || runtime.jobs.has_pending(&projects).await?;
    for kind in &input.types {
      let mut documents = Vec::new();
      let mut seen = HashSet::new();
      if !published.is_empty() {
        let page = runtime
          .engine()?
          .query(EngineQuery {
            query: input.query.clone(),
            project_ids: projects.clone(),
            kind: (*kind).into(),
            confidential_projects: scope.confidential_projects.clone(),
            confidential_issue_ids: scope.confidential_ids.clone(),
            namespaces: published
              .iter()
              .map(|(id, token)| (*id, token.clone()))
              .collect(),
            offset: 0,
            limit: CANDIDATE_BUDGET,
          })
          .await
          .map_err(|error| {
            tracing::warn!(error=%error,"global search engine request failed");
            unavailable()
          })?;
        partial |= page.partial || page.has_more;
        for mut doc in page.documents {
          if seen.insert(doc.key.clone()) {
            doc.body = String::new();
            doc.title = String::new();
            documents.push(doc);
          }
        }
      }
      groups.push(Group {
        kind: *kind,
        documents,
        offset: 0,
        returned_files: Default::default(),
      });
    }
    Session {
      user: scope.actor.user_id,
      scope: scope.fingerprint.clone(),
      request,
      expires: Instant::now() + Duration::from_secs(120),
      groups,
      partial,
    }
  };
  let issue_targets = session
    .groups
    .iter()
    .filter(|g| g.kind == SearchKind::Issue)
    .flat_map(|g| g.documents.iter().skip(g.offset))
    .filter_map(|d| {
      Some((
        d.project_id,
        "issue".to_owned(),
        d.resource_id.parse::<i64>().ok()?,
      ))
    })
    .collect::<Vec<_>>();
  let issues = s.collaboration.issue_targets(&issue_targets).await?;
  let mut groups = Vec::new();
  for group in &mut session.groups {
    let mut items = Vec::new();
    while group.offset < group.documents.len() && items.len() < input.limit as usize {
      let doc = &group.documents[group.offset];
      group.offset += 1;
      let file_key = format!("{}:{}:{}", doc.project_id, doc.version, doc.path);
      if group.kind == SearchKind::Code && group.returned_files.contains(&file_key) {
        continue;
      }
      if let Some(hit) =
        super::hydrate::hit(s, &scope, doc, &input.query, group.kind, &issues).await?
      {
        if group.kind == SearchKind::Code {
          group.returned_files.insert(file_key);
        }
        items.push(hit);
      }
    }
    groups.push(SearchGroup {
      kind: group.kind,
      items,
    });
  }
  // Revalidate the token and membership after engine I/O and hydration. Never
  // trust an old cursor, cached permission set, or an indexed visibility flag.
  let final_scope = scope::resolve(s, h, &input).await?;
  if final_scope.fingerprint != scope.fingerprint {
    return Err(AppError::conflict("search scope changed; restart search"));
  }
  super::hydrate::revalidate(s, &final_scope, &mut groups, &issues).await?;
  let next_cursor = if session.groups.iter().any(|g| g.offset < g.documents.len()) {
    let cursor = uuid::Uuid::now_v7().to_string();
    let mut sessions = runtime.sessions.lock().map_err(|_| unavailable())?;
    sessions.retain(|_, v| v.expires > Instant::now());
    let bytes = session.estimated_bytes();
    const CACHE_BYTES: usize = 64 * 1024 * 1024;
    if bytes > CACHE_BYTES {
      return Err(unavailable());
    }
    while sessions.len() >= 128
      || sessions
        .values()
        .map(Session::estimated_bytes)
        .sum::<usize>()
        + bytes
        > CACHE_BYTES
      || sessions.values().filter(|v| v.user == session.user).count() >= 8
    {
      let user_full = sessions.values().filter(|v| v.user == session.user).count() >= 8;
      if let Some(key) = sessions
        .iter()
        .filter(|(_, v)| !user_full || v.user == session.user)
        .min_by_key(|(_, v)| v.expires)
        .map(|(k, _)| k.clone())
      {
        sessions.remove(&key);
      } else {
        break;
      }
    }
    sessions.insert(cursor.clone(), session.clone());
    Some(cursor)
  } else {
    None
  };
  let project_ids = scope
    .projects
    .iter()
    .map(|p| p.project.id)
    .collect::<Vec<_>>();
  let pending = runtime.jobs.has_pending(&project_ids).await?;
  Ok(SearchReply {
    groups,
    next_cursor,
    partial: session.partial || pending,
    indexing_eventually_consistent: true,
    max_code_files_per_project: s.settings.search.max_files_per_project,
    max_code_file_size_bytes: s.settings.search.max_file_size_bytes,
  })
}
