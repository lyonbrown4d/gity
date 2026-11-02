pub(super) use super::merge_views::{flag, milestone_patch, view};
pub(super) use super::project_lock::lock;
use super::*;
const MR: &str = "project_merge_requests";
use crate::openapi::{merge_schemas as spec, schemas as api};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  let shared = utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list, create))
    .routes(utoipa_axum::routes!(detail, update))
    .routes(utoipa_axum::routes!(diff))
    .routes(utoipa_axum::routes!(checks))
    .routes(utoipa_axum::routes!(merge))
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
#[utoipa::path(
    get,path="/{id}/merge-requests",tag="merge requests",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::MergeRequest>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  let rows = s
    .db
    .all(
      "SELECT * FROM project_merge_requests WHERE project_id = ? ORDER BY iid DESC",
      vec![p.into()],
    )
    .await?;
  response(json!(rows.into_iter().map(view).collect::<Vec<_>>()))
}
#[utoipa::path(
    get,path="/{id}/merge-requests/{iid}",tag="merge requests",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::MergeRequest>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn detail(State(s): State<AppState>, h: HeaderMap, Path((p, i)): Path<(i64, i64)>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(scoped(&s, MR, p, i, true).await?)
}
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
  let iid = s.db.next_iid(p, "merge_request").await?;
  let values = json!({
    "project_id":p,"iid":iid,"author_user_id":actor.user_id,"title":title,
    "description":string(&b,"description"),"state":"opened","source_branch":source,
    "target_branch":target,"head_sha":head_sha,
    "milestone_id":milestone_patch(&s, p, &b, "milestone_id").await?,
    "squash":flag(&b,"squash"),"draft":flag(&b,"draft"),
  });
  response(view(s.db.insert(MR, fields(values)).await?))
}
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
  let row = scoped(&s, MR, p, i, true).await?;
  if b.get("title").is_some() {
    required(&b, "title")?;
  }
  let mut changes = patch(&b, &["title", "description", "state"]);
  if let Some(state) = b.get("state") {
    if state != "opened" && state != "closed" {
      return Err(AppError::bad_request(
        "use merge action to merge a merge request",
      ));
    }
    if string(&row, "state") == "merged" {
      return Err(AppError::conflict(
        "merged merge requests cannot be reopened",
      ));
    }
    // Closing stamps the time; reopening clears it again.
    let closed_at = (state == "closed").then(now);
    changes.insert("closed_at".into(), json!(closed_at));
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
  response(view(s.db.update(MR, number(&row, "id"), changes).await?))
}
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
  let row = scoped(&s, MR, p, i, true).await?;
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
  let row = scoped(&s, MR, p, i, true).await?;
  response(super::checks::evaluate(&s, &project, &row).await?)
}
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
  let authorization =
    crate::access::project_action(&s, &h, p, ProjectAction::MergeRequestMerge).await?;
  let actor = authorization.actor()?;
  let project = authorization.project;
  let _guard = lock(p).await;
  let row = scoped(&s, MR, p, i, true).await?;
  if string(&row, "state") != "opened" {
    return Err(AppError::conflict("merge request is not opened"));
  }
  let checks = super::checks::evaluate(&s, &project, &row).await?;
  if checks["mergeable"] != true {
    return Err(AppError::conflict(string(&checks, "blocking_reason")));
  }
  if string(&checks, "merge_access_level") == "owner" {
    crate::access::project(&s, &h, p, 50).await?;
  }
  if string(&checks, "merge_access_level") == "no_one" {
    return Err(AppError::forbidden("target branch does not permit merges"));
  }
  let message = if string(&b, "message").is_empty() {
    format!(
      "Merge branch '{}' into '{}'",
      string(&row, "source_branch"),
      string(&row, "target_branch")
    )
  } else {
    string(&b, "message").to_owned()
  };
  s.git
    .merge_branches(
      &repository(&project),
      gity_git::MergeBranchesInput {
        target_branch: string(&row, "target_branch").into(),
        source_branch: string(&row, "source_branch").into(),
        message,
        author_name: if string(&b, "author_name").is_empty() {
          actor.username.clone()
        } else {
          string(&b, "author_name").into()
        },
        author_email: if string(&b, "author_email").is_empty() {
          format!("{}@gity.local", actor.username)
        } else {
          string(&b, "author_email").into()
        },
        expected_source_sha: Some(string(&checks, "source_commit_sha").into()),
      },
    )
    .await
    .map_err(|e| AppError::conflict(e.to_string()))?;
  // The target tip is the merge commit; without git proof the sha stays unset.
  let target = string(&row, "target_branch");
  let default = string(&project, "default_branch");
  let sha = s
    .git
    .resolve_revision(&repository(&project), target, default)
    .await
    .map_err(|error| tracing::warn!(project_id=p,error=%error,"merged commit sha unavailable"))
    .ok();
  let result = s
    .db
    .update(
      MR,
      number(&row, "id"),
      fields(json!({
          "state":"merged","merged_at":now(),"merged_by_user_id":actor.user_id,
          "merge_commit_sha":sha,
      })),
    )
    .await?;
  super::merge_events::merged(&s, &project, &row, &actor).await?;
  response(view(result))
}
