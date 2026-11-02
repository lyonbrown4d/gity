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
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Issue>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::Read).await?;
  let rows = s
    .collaboration
    .issues(p)
    .await?
    .into_iter()
    .map(model_value)
    .collect();
  response(json!(super::issue_views::list(&s, p, rows).await?))
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
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
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

  let milestone_id = milestone_ref(&s, p, b.get("milestone_id")).await?;
  let due_date = date_value(b.get("due_date"))?;
  let weight = weight_value(b.get("weight"))?;
  let row = model_value(
    s.collaboration
      .create_issue(IssueCreate {
        project_id: p,
        author_user_id: actor.user_id,
        title: title.clone(),
        description: Some(string(&b, "description").to_owned()),
        milestone_id,
        due_date,
        weight,
        confidential: flag(&b),
        issue_type: kind(&b),
      })
      .await?,
  );
  let iid = number(&row, "iid");
  let recipients = s.collaboration.recipients(p).await?;
  for u in recipients {
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
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
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
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::IssueWrite)
    .await?
    .actor()?;
  s.collaboration.issue(p, i).await?;
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
  let mut change = patch(
    &b,
    &[
      "title",
      "description",
      "state",
      "milestone_id",
      "due_date",
      "weight",
      "confidential",
      "issue_type",
    ],
  );
  change.insert("updated_by_user_id".into(), json!(actor.user_id));
  if b.get("milestone_id").is_some() {
    change.insert(
      "milestone_id".into(),
      json!(milestone_ref(&s, p, b.get("milestone_id")).await?),
    );
  }
  if b.get("due_date").is_some() {
    change.insert("due_date".into(), json!(date_value(b.get("due_date"))?));
  }
  if b.get("weight").is_some() {
    change.insert("weight".into(), json!(weight_value(b.get("weight"))?));
  }
  if b.get("confidential").is_some() {
    change.insert("confidential".into(), json!(flag(&b)));
  }
  if b.get("issue_type").is_some() {
    change.insert("issue_type".into(), json!(kind(&b)));
  }
  let row = model_value(
    s.collaboration
      .update_issue(p, i, command(Value::Object(change))?)
      .await?,
  );
  response(super::issue_views::one(&s, p, row).await?)
}
async fn milestone_ref(
  s: &AppState,
  project: i64,
  value: Option<&Value>,
) -> Result<Option<i64>, AppError> {
  let Some(value) = value else {
    return Ok(None);
  };
  if value.is_null() {
    return Ok(None);
  }
  let id = value
    .as_i64()
    .or_else(|| value.as_str().and_then(|v| v.parse().ok()))
    .ok_or_else(|| AppError::bad_request("milestone_id must be an integer"))?;
  s.collaboration.milestone_id(project, id).await.map(Some)
}
fn date_value(value: Option<&Value>) -> Result<Option<chrono::NaiveDate>, AppError> {
  match value {
    None | Some(Value::Null) => Ok(None),
    Some(Value::String(raw)) => {
      let raw = raw.trim();
      if raw.is_empty() {
        return Ok(None);
      }
      chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| AppError::bad_request("due_date must use YYYY-MM-DD"))
    }
    Some(_) => Err(AppError::bad_request("due_date must use YYYY-MM-DD")),
  }
}
fn weight_value(value: Option<&Value>) -> Result<Option<i64>, AppError> {
  match value {
    None | Some(Value::Null) => Ok(None),
    Some(value) => value
      .as_i64()
      .or_else(|| value.as_str().and_then(|v| v.parse().ok()))
      .filter(|v| *v > 0)
      .map(Some)
      .ok_or_else(|| AppError::bad_request("weight must be a positive integer")),
  }
}
fn flag(b: &Value) -> i64 {
  match b.get("confidential") {
    Some(Value::Bool(value)) => i64::from(*value),
    Some(Value::String(value)) => i64::from(value == "true" || value == "1"),
    Some(Value::Number(value)) => i64::from(value.as_i64().unwrap_or(0) != 0),
    _ => 0,
  }
}
fn kind(b: &Value) -> String {
  match string(b, "issue_type").trim() {
    "" => "issue".to_owned(),
    value => value.to_owned(),
  }
}
