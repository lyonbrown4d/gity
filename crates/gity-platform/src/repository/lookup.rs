//! Read-only code intelligence: outline, definition, references and hover.
//!
//! Answers come from the project's lexical index. Missing or unreadable indexes
//! are not errors — they produce empty results with `precision = "approximate"`,
//! which is what lets the precise layer be added later without a shape change.
use crate::access::{self, ProjectAction};
use crate::openapi::code_schemas as spec;
use crate::openapi::schemas as api;
use crate::{AppState, repository};
use axum::{
  Json,
  extract::{Path, Query, State},
  http::HeaderMap,
};
use gity_code_intel::{Status, SymbolDoc};
use gity_core::{AppError, ok};
use serde_json::{Value, json};
use std::collections::HashMap;
type Reply = Result<Json<Value>, AppError>;

pub(crate) fn routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(symbols))
    .routes(utoipa_axum::routes!(definitions))
    .routes(utoipa_axum::routes!(references))
    .routes(utoipa_axum::routes!(hover))
    .routes(utoipa_axum::routes!(search))
}

/// Resolves a path, line and column triple with the shared validation rules.
fn position(query: &HashMap<String, String>) -> Result<(String, u64, u64), AppError> {
  let path = query
    .get("path")
    .filter(|path| !path.is_empty())
    .ok_or_else(|| AppError::bad_request("path is required"))?
    .clone();
  let number = |key: &str| -> Result<u64, AppError> {
    query
      .get(key)
      .ok_or_else(|| AppError::bad_request(format!("{key} is required")))?
      .parse::<u64>()
      .ok()
      .filter(|value| *value >= 1)
      .ok_or_else(|| AppError::bad_request(format!("{key} must be a positive integer")))
  };
  Ok((path, number("line")?, number("column")?))
}

fn limit(query: &HashMap<String, String>, default: usize) -> Result<usize, AppError> {
  match query.get("limit") {
    None => Ok(default),
    Some(raw) => raw
      .parse::<usize>()
      .ok()
      .filter(|value| (1..=500).contains(value))
      .ok_or_else(|| AppError::bad_request("limit must be between 1 and 500")),
  }
}

async fn blocking<T: Send + 'static>(
  action: impl FnOnce() -> anyhow::Result<T> + Send + 'static,
) -> Result<T, AppError> {
  tokio::task::spawn_blocking(action)
    .await
    .map_err(|_| AppError::internal("code index worker task failed"))?
    .map_err(|error| AppError::internal(format!("code index lookup failed: {error}")))
}

/// Pulls the shared archive when this replica has no local index for the commit.
async fn restore(s: &AppState, project: &Value) {
  crate::code_index::hydrate_head(s, project).await;
}

fn index_info(status: Option<&Status>) -> spec::CodeIndexInfo {
  spec::CodeIndexInfo {
    available: status.is_some(),
    commit: status
      .map(|status| status.commit.clone())
      .unwrap_or_default(),
    indexed_at: status
      .map(|status| status.indexed_at.clone())
      .unwrap_or_default(),
    truncated: status.map(|status| status.truncated).unwrap_or(false),
  }
}

fn symbol(row: SymbolDoc) -> spec::CodeSymbol {
  spec::CodeSymbol {
    path: row.path,
    name: row.name,
    kind: row.kind,
    scope: row.scope,
    line: row.line,
    line_end: row.line_end,
    column: row.column,
    doc: row.doc,
  }
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(
    get, path="/symbols", tag="Repository",
    summary="List the declarations of one file as an outline",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::CodeSymbols>),api::Errors),
    security((),("bearer"=[]),("private_token"=[]))
)]
pub(crate) async fn symbols(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  let project = access::project_action(&s, &h, id, ProjectAction::Read)
    .await?
    .project;
  restore(&s, &project).await;
  let path = q
    .get("path")
    .filter(|path| !path.is_empty())
    .ok_or_else(|| AppError::bad_request("path is required"))?
    .clone();
  let store = s.code_index.clone();
  let lookup = path.clone();
  let (status, rows) = blocking(move || {
    let status = store.status(id)?;
    let rows = store.symbols(id, &lookup)?;
    anyhow::Ok((status, rows))
  })
  .await?;
  Ok(ok(json!(spec::CodeSymbols {
    path,
    items: rows.into_iter().map(symbol).collect(),
    precision: "approximate".into(),
    index: index_info(status.as_ref()),
  })))
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(
    get, path="/definitions", tag="Repository",
    summary="Resolve the declaration covering a position",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::CodeDefinitions>),api::Errors),
    security((),("bearer"=[]),("private_token"=[]))
)]
pub(crate) async fn definitions(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  let project = access::project_action(&s, &h, id, ProjectAction::Read)
    .await?
    .project;
  restore(&s, &project).await;
  let (path, line, column) = position(&q)?;
  let store = s.code_index.clone();
  let (status, found) = blocking(move || {
    let status = store.status(id)?;
    let found = store.definition_at(id, &path, line, column)?;
    anyhow::Ok((status, found))
  })
  .await?;
  Ok(ok(json!(spec::CodeDefinitions {
    definitions: found.into_iter().map(symbol).collect(),
    mode: "enclosing".into(),
    precision: "approximate".into(),
    index: index_info(status.as_ref()),
  })))
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(
    get, path="/references", tag="Repository",
    summary="List the declarations of a name in the indexed snapshot",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::CodeReferences>),api::Errors),
    security((),("bearer"=[]),("private_token"=[]))
)]
pub(crate) async fn references(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  let project = access::project_action(&s, &h, id, ProjectAction::Read)
    .await?
    .project;
  restore(&s, &project).await;
  let name = q
    .get("name")
    .filter(|name| !name.is_empty())
    .ok_or_else(|| AppError::bad_request("name is required"))?
    .clone();
  let limit = limit(&q, 100)?;
  let store = s.code_index.clone();
  let (status, rows) = blocking(move || {
    let status = store.status(id)?;
    let rows = store.references(id, &name)?;
    anyhow::Ok((status, rows))
  })
  .await?;
  let total = rows.len() as u64;
  let references = rows
    .into_iter()
    .take(limit)
    .map(|row| spec::CodeReference {
      path: row.path,
      name: row.name,
      line: row.line,
      column: row.column,
      scope: row.scope,
    })
    .collect();
  Ok(ok(json!(spec::CodeReferences {
    references,
    total,
    precision: "approximate".into(),
    index: index_info(status.as_ref()),
  })))
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(
    get, path="/hover", tag="Repository",
    summary="Return the documentation of the declaration covering a position",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::CodeHover>),api::Errors),
    security((),("bearer"=[]),("private_token"=[]))
)]
pub(crate) async fn hover(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  let project = access::project_action(&s, &h, id, ProjectAction::Read)
    .await?
    .project;
  restore(&s, &project).await;
  let (path, line, column) = position(&q)?;
  let store = s.code_index.clone();
  let (status, found) = blocking(move || {
    let status = store.status(id)?;
    let found = store.hover_at(id, &path, line, column)?;
    anyhow::Ok((status, found))
  })
  .await?;
  let hover = found.map(|row| spec::CodeHoverContent {
    contents_markdown: if row.doc.is_empty() {
      format!("`{}` ({})", row.name, row.kind)
    } else {
      format!("{}\n\n```\n{}\n```", row.doc, row.name)
    },
    kind: row.kind,
    source: "lexical".into(),
  });
  Ok(ok(json!(spec::CodeHover {
    hover,
    precision: "approximate".into(),
    index: index_info(status.as_ref()),
  })))
}

#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(
    get, path="/code-search", tag="Repository",
    summary="Search declaration names and paths in the indexed snapshot",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::CodeSearchResults>),api::Errors),
    security((),("bearer"=[]),("private_token"=[]))
)]
pub(crate) async fn search(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  let project = access::project_action(&s, &h, id, ProjectAction::Read)
    .await?
    .project;
  restore(&s, &project).await;
  let text = q
    .get("q")
    .or_else(|| q.get("query"))
    .filter(|text| !text.is_empty())
    .ok_or_else(|| AppError::bad_request("q is required"))?
    .clone();
  let limit = limit(&q, 50)?;
  let store = s.code_index.clone();
  let (status, rows) = blocking(move || {
    let status = store.status(id)?;
    let rows = store.search(id, &text, limit)?;
    anyhow::Ok((status, rows))
  })
  .await
  .map_err(|error| AppError::bad_request(format!("invalid search query: {error}")))?;
  Ok(ok(json!(spec::CodeSearchResults {
    total: rows.len() as u64,
    matches: rows.into_iter().map(symbol).collect(),
    precision: "approximate".into(),
    index: index_info(status.as_ref()),
  })))
}

/// Kept so the module compiles when the route table is trimmed during reviews.
#[allow(dead_code)]
fn _project_repo(project: &Value) -> String {
  repository::repo(project)
}
