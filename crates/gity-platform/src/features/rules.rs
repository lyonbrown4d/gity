use super::*;
use crate::openapi::{merge_schemas as spec, schemas as api};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list, create))
    .routes(utoipa_axum::routes!(update, remove))
}
pub(super) fn ids(row: &Value) -> Vec<i64> {
  string(row, "eligible_user_ids")
    .split(|c: char| c == ',' || c.is_whitespace())
    .filter_map(|v| v.parse().ok())
    .filter(|v| *v > 0)
    .collect()
}
pub(super) fn view(mut row: Value) -> Value {
  let ids = ids(&row)
    .into_iter()
    .map(|v| v.to_string())
    .collect::<Vec<_>>();
  row["eligible_user_ids"] = json!(ids);
  row["code_owner"] = json!(number(&row, "code_owner") != 0);
  row
}
async fn normalize(
  s: &AppState,
  b: &Value,
  creating: bool,
) -> Result<Map<String, Value>, AppError> {
  let mut out = super::patch(
    b,
    &["name", "target_branch", "approvals_required", "code_owner"],
  );
  if creating || b.get("name").is_some() {
    out.insert("name".into(), json!(required(b, "name")?));
  }
  if creating || b.get("target_branch").is_some() {
    let value = string(b, "target_branch").trim();
    out.insert(
      "target_branch".into(),
      json!(if value.is_empty() { "*" } else { value }),
    );
  }
  if creating || b.get("approvals_required").is_some() {
    out.insert(
      "approvals_required".into(),
      json!(number(b, "approvals_required").max(1)),
    );
  }
  if creating || b.get("code_owner").is_some() {
    out.insert(
      "code_owner".into(),
      json!(if b["code_owner"] == true { 1 } else { 0 }),
    );
  }
  if creating || b.get("eligible_user_ids").is_some() {
    let users = users(s, b.get("eligible_user_ids").unwrap_or(&json!([]))).await?;
    out.insert(
      "eligible_user_ids".into(),
      json!(
        users
          .into_iter()
          .map(|v| v.to_string())
          .collect::<Vec<_>>()
          .join(",")
      ),
    );
  }
  Ok(out)
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    get,path="/projects/{id}/merge-request-approval-rules",tag="rules",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ApprovalRules>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin).await?;
  response(
    json!({"project_id":p,"rules":s.collaboration.approval_rules(p).await?.into_iter().map(model_value).map(view).collect::<Vec<_>>()}),
  )
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    post,path="/projects/{id}/merge-request-approval-rules",tag="rules",
    params(api::ProjectPath),
    request_body(content=spec::CreateApprovalRule),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ApprovalRule>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn create(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin)
    .await?
    .actor()?
    .user_id;
  let _guard = super::merge_requests::lock(p).await;
  let mut data = normalize(&s, &b, true).await?;
  data.insert("project_id".into(), json!(p));
  response(view(model_value(
    s.collaboration
      .create_approval_rule(command(Value::Object(data))?, Some(actor))
      .await?,
  )))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    patch,path="/projects/{id}/merge-request-approval-rules/{rule}",tag="rules",
    params(api::RulePath),
    request_body(content=spec::UpdateApprovalRule),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ApprovalRule>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn update(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, r)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin)
    .await?
    .actor()?
    .user_id;
  let _guard = super::merge_requests::lock(p).await;
  s.collaboration.approval_rule(p, r).await?;
  response(view(model_value(
    s.collaboration
      .update_approval_rule(
        p,
        r,
        command(Value::Object(normalize(&s, &b, false).await?))?,
        Some(actor),
      )
      .await?,
  )))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    delete,path="/projects/{id}/merge-request-approval-rules/{rule}",tag="rules",
    params(api::RulePath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<api::Deleted>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn remove(State(s): State<AppState>, h: HeaderMap, Path((p, r)): Path<(i64, i64)>) -> Reply {
  let actor = crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin)
    .await?
    .actor()?
    .user_id;
  let _guard = super::merge_requests::lock(p).await;
  s.collaboration.approval_rule(p, r).await?;
  s.collaboration
    .delete_approval_rule(p, r, Some(actor))
    .await?;
  response(json!({"deleted":true}))
}
