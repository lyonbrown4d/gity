pub(super) use super::merge_views::{flag, milestone_patch, view};
pub(super) use super::project_lock::lock;
use super::*;
use crate::openapi::{merge_schemas as spec, schemas as api};
use axum::{
  extract::{Path, Query, State},
  http::HeaderMap,
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  let shared = utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list, create))
    .routes(utoipa_axum::routes!(detail, update))
    .routes(utoipa_axum::routes!(diff))
    .routes(utoipa_axum::routes!(checks))
    .routes(utoipa_axum::routes!(merge))
    .routes(utoipa_axum::routes!(execution::abort))
    .routes(utoipa_axum::routes!(
      super::mr_collaboration::comments,
      super::mr_collaboration::comment
    ))
    .routes(utoipa_axum::routes!(super::mr_collaboration::participants))
    .routes(utoipa_axum::routes!(super::mr_collaboration::reviewers))
    .routes(utoipa_axum::routes!(super::mr_collaboration::assignees))
    .routes(utoipa_axum::routes!(super::mr_collaboration::approvals))
    .routes(utoipa_axum::routes!(super::mr_collaboration::approve))
    .routes(utoipa_axum::routes!(super::mr_collaboration::unapprove));
  utoipa_axum::router::OpenApiRouter::new()
    .nest("/projects", shared.clone())
    .nest("/repos", shared)
}
pub(super) fn git_error(e: anyhow::Error) -> AppError {
  AppError::bad_request(e.to_string())
}
pub(super) fn repository(p: &Value) -> String {
  crate::repository::repo(p)
}
pub(super) async fn branch(s: &AppState, p: &Value, name: &str) -> Result<String, AppError> {
  let rows = s
    .git
    .list_branches(&repository(p), string(p, "default_branch"))
    .await
    .map_err(git_error)?;
  rows
    .as_array()
    .and_then(|v| v.iter().find(|v| string(v, "name") == name))
    .map(|v| string(v, "hash").to_owned())
    .ok_or_else(|| AppError::not_found("merge request branch not found"))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get,path="/{id}/merge-requests",tag="merge requests",
    description="Lists merge requests newest first; `state` filters by state and `page`/`page_size` paginate.",
    params(api::ProjectPath, api::ListQuery),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::MergeRequestPage>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn list(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Query(params): Query<api::ListQuery>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  let state = params.state.as_deref().unwrap_or("all");
  if !matches!(state, "opened" | "merged" | "closed" | "all") {
    return Err(AppError::bad_request(
      "state must be opened, merged, closed or all",
    ));
  }
  let (page, page_size, offset) = super::paging::window(&params)?;
  let rows = s
    .collaboration
    .merge_requests(p)
    .await?
    .into_iter()
    .map(model_value)
    .filter(|row| {
      state == "all" || row["state"].as_str().or_else(|| row["status"].as_str()) == Some(state)
    })
    .collect::<Vec<_>>();
  let total = rows.len() as u64;
  let items = rows
    .into_iter()
    .skip(offset)
    .take(page_size as usize)
    .map(view)
    .collect::<Vec<_>>();
  response(json!({
      "items": items,
      "total": total,
      "page": page,
      "page_size": page_size,
  }))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get,path="/{id}/merge-requests/{iid}",tag="merge requests",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::MergeRequest>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn detail(State(s): State<AppState>, h: HeaderMap, Path((p, i)): Path<(i64, i64)>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(model_value(&s.collaboration.merge_request(p, i).await?))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    post,path="/{id}/merge-requests",tag="merge requests",
    params(api::ProjectPath),
    request_body(content=spec::CreateMergeRequest),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::MergeRequest>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn create(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  let authorization =
    crate::access::project_action(&s, &h, p, ProjectAction::MergeRequestCreate).await?;
  let actor = authorization.actor()?;
  let project = authorization.project;
  let title = required(&b, "title")?;
  let source = required(&b, "source_branch")?;
  let target = required(&b, "target_branch")?;
  if source == target {
    return Err(AppError::bad_request(
      "source_branch and target_branch must be different",
    ));
  }
  let head_sha = branch(&s, &project, &source).await?;
  branch(&s, &project, &target).await?;
  let milestone = milestone_patch(&s, p, &b, "milestone_id").await?;
  response(view(model_value(
    s.collaboration
      .create_merge_request(MergeRequestCreate {
        project_id: p,
        author_user_id: actor.user_id,
        title,
        description: Some(string(&b, "description").to_owned()),
        source_branch: source,
        target_branch: target,
        head_sha: Some(head_sha),
        milestone_id: milestone.as_i64(),
        squash: flag(&b, "squash"),
        draft: flag(&b, "draft"),
      })
      .await?,
  )))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    patch,path="/{id}/merge-requests/{iid}",tag="merge requests",
    params(api::IidPath),
    request_body(content=spec::UpdateMergeRequest),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::MergeRequest>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn update(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::MergeRequestWrite).await?;
  let _guard = lock(p).await;
  s.collaboration.merge_request(p, i).await?;
  if b.get("title").is_some() {
    required(&b, "title")?;
  }
  let mut changes = patch(&b, &["title", "description", "state"]);
  if let Some(state) = b.get("state")
    && state != "opened"
    && state != "closed"
  {
    return Err(AppError::bad_request(
      "use merge action to merge a merge request",
    ));
  }
  for key in ["squash", "draft"] {
    if let Some(value) = b.get(key).and_then(Value::as_bool) {
      changes.insert(key.into(), json!(i64::from(value)));
    }
  }
  if b.get("milestone_id").is_some() {
    changes.insert(
      "milestone_id".into(),
      milestone_patch(&s, p, &b, "milestone_id").await?,
    );
  }
  response(view(model_value(
    s.collaboration
      .update_merge_request(p, i, command(Value::Object(changes))?)
      .await?,
  )))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get,path="/{id}/merge-requests/{iid}/diff",tag="merge requests",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::MergeDiff>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn diff(State(s): State<AppState>, h: HeaderMap, Path((p, i)): Path<(i64, i64)>) -> Reply {
  let project = crate::access::project_action(&s, &h, p, ProjectAction::Read)
    .await?
    .project;
  let row = model_value(&s.collaboration.merge_request(p, i).await?);
  let diff = s
    .git
    .diff_branches(
      &repository(&project),
      string(&row, "target_branch"),
      string(&row, "source_branch"),
    )
    .await
    .map_err(git_error)?;
  response(json!({"merge_request":view(row),"diff":diff}))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get,path="/{id}/merge-requests/{iid}/checks",tag="merge requests",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::MergeChecks>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn checks(State(s): State<AppState>, h: HeaderMap, Path((p, i)): Path<(i64, i64)>) -> Reply {
  let project = crate::access::project_action(&s, &h, p, ProjectAction::Read)
    .await?
    .project;
  let row = model_value(&s.collaboration.merge_request(p, i).await?);
  response(super::checks::evaluate(&s, &project, &row).await?)
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    post,path="/{id}/merge-requests/{iid}/merge",tag="merge requests",
    params(api::IidPath),
    request_body(content=spec::MergeCommand),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::MergeRequest>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn merge(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  execution::merge(&s, &h, p, i, &b).await
}
#[path = "merge_execution.rs"]
mod execution;
