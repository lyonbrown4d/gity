use super::*;
use crate::openapi::{issue_schemas as spec, schemas as api};
use axum::{
  extract::{Path, Query, State},
  http::HeaderMap,
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  let shared = utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list))
    .routes(utoipa_axum::routes!(detail))
    .merge(super::issues_mutation::documented_routes())
    .routes(utoipa_axum::routes!(
      super::issue_collaboration::comments,
      super::issue_collaboration::comment
    ))
    .routes(utoipa_axum::routes!(
      super::issue_collaboration::assignees,
      super::issue_collaboration::set_assignees
    ))
    .routes(utoipa_axum::routes!(
      super::issue_collaboration::labels,
      super::issue_collaboration::set_labels
    ));
  let mut router = utoipa_axum::router::OpenApiRouter::new()
    .nest("/projects", shared.clone())
    .nest("/repos", shared);
  crate::openapi::alias(
    &mut router,
    "/repos/{id}/issues/{iid}",
    "/repos/{id}/issues/by-number/{iid}",
  );
  router = router.route(
    "/repos/{id}/issues/by-number/{iid}",
    axum::routing::get(detail),
  );
  router
}
pub(super) fn view(row: Value) -> Value {
  json!({
      "id":number(&row, "iid").to_string(),
      "repository_id":number(&row, "project_id").to_string(),
      "number":row["iid"],
      "title":row["title"],
      "assignee_user_id":Value::Null,
      "assignee_user_ids":[],
      "assignees":[],
      "labels":[],
      "description":string(&row, "description"),
      "status":if string(&row, "state")=="closed" {"closed"} else {"open"},
      "author_user_id":row["author_user_id"],
      "milestone_id":row["milestone_id"],
      "due_date":row["due_date"],
      "weight":row["weight"],
      "confidential":number(&row, "confidential") != 0,
      "issue_type":if string(&row, "issue_type").is_empty() {"issue"} else {string(&row, "issue_type")},
      "closed_by_user_id":row["closed_by_user_id"],
      "closed_at":row["closed_at"],
      "created_at":row["created_at"],
      "updated_at":row["updated_at"],
  })
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get,path="/{id}/issues",tag="issues",
    description="Lists issues newest first; `state` filters by status and `page`/`page_size` paginate.",
    params(api::ProjectPath, api::ListQuery),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::IssuePage>),api::Errors),
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
  if !matches!(state, "open" | "closed" | "all") {
    return Err(AppError::bad_request("state must be open, closed or all"));
  }
  let (page, page_size, offset) = super::paging::window(&params)?;
  let rows = s
    .collaboration
    .issues(p)
    .await?
    .into_iter()
    .map(model_value)
    .filter(|row| {
      // Stored issues use `opened` while the API exposes them as `open`.
      let raw = row["status"].as_str().or_else(|| row["state"].as_str());
      state == "all"
        || match raw {
          Some("open" | "opened") => state == "open",
          Some(value) => value == state,
          None => false,
        }
    })
    .collect::<Vec<_>>();
  let total = rows.len() as u64;
  let window = rows
    .into_iter()
    .skip(offset)
    .take(page_size as usize)
    .collect::<Vec<_>>();
  let items = super::issue_views::list(&s, p, window).await?;
  response(json!({
      "items": items,
      "total": total,
      "page": page,
      "page_size": page_size,
  }))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get,path="/{id}/issues/{iid}",tag="issues",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Issue>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn detail(State(s): State<AppState>, h: HeaderMap, Path((p, i)): Path<(i64, i64)>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(super::issue_views::one(&s, p, model_value(&s.collaboration.issue(p, i).await?)).await?)
}
