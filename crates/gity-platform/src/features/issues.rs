use super::*;
use crate::openapi::{issue_schemas as spec, schemas as api};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  let shared = utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list, create))
    .routes(utoipa_axum::routes!(detail, update))
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
      "created_at":row["created_at"],
      "updated_at":row["updated_at"],
  })
}
#[utoipa::path(
    get,path="/{id}/issues",tag="issues",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Issue>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  let rows = s
    .db
    .all(
      "SELECT * FROM project_issues WHERE project_id = ? ORDER BY iid DESC",
      vec![p.into()],
    )
    .await?;
  response(json!(super::issue_views::list(&s, p, rows).await?))
}
#[utoipa::path(
    get,path="/{id}/issues/{iid}",tag="issues",
    params(api::IidPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Issue>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn detail(State(s): State<AppState>, h: HeaderMap, Path((p, i)): Path<(i64, i64)>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  response(super::issue_views::one(&s, p, scoped(&s, "project_issues", p, i, true).await?).await?)
}
#[utoipa::path(
    post,path="/{id}/issues",tag="issues",
    params(api::ProjectPath),
    request_body(content=spec::CreateIssue),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Issue>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn create(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::IssueCreate)
    .await?
    .actor()?;
  let title = required(&b, "title")?;
  let iid = s.db.next_iid(p, "issue").await?;
  let row=s.db.insert("project_issues",fields(json!({"project_id":p,"iid":iid,"author_user_id":actor.user_id,"title":title,"description":string(&b,"description"),"state":"opened"}))).await?;
  let recipients=s.db.all("SELECT user_id FROM project_members WHERE project_id = ? UNION SELECT user_id FROM organization_members WHERE organization_id = (SELECT organization_id FROM projects WHERE id = ?)",vec![p.into(),p.into()]).await?;
  for user in recipients {
    let u = number(&user, "user_id");
    if u != actor.user_id {
      super::todos::notify(
        &s,
        super::todos::Notification {
          project: p,
          user: u,
          kind: "issue_created",
          target_type: "issue",
          target: number(&row, "id"),
          title: &format!("Issue #{iid} opened"),
          summary: &title,
          suffix: &format!("/issues/{iid}"),
        },
      )
      .await?;
    }
  }
  response(view(row))
}
#[utoipa::path(
    patch,path="/{id}/issues/{iid}",tag="issues",
    params(api::IidPath),
    request_body(content=spec::UpdateIssue),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Issue>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn update(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
  Json(mut b): Json<Value>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::IssueWrite).await?;
  let row = scoped(&s, "project_issues", p, i, true).await?;
  if b.get("title").is_some() {
    required(&b, "title")?;
  }
  if b.get("state").is_none()
    && let Some(v) = b.get("status").cloned()
  {
    b["state"] = json!(if v == "open" {
      "opened"
    } else {
      v.as_str().unwrap_or("")
    });
  }
  if let Some(v) = b.get("state")
    && v != "opened"
    && v != "closed"
  {
    return Err(AppError::bad_request(
      "issue state must be opened or closed",
    ));
  }
  let row = s
    .db
    .update(
      "project_issues",
      number(&row, "id"),
      patch(&b, &["title", "description", "state"]),
    )
    .await?;
  response(super::issue_views::one(&s, p, row).await?)
}
