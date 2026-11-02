use super::*;
use crate::openapi::{credential_schemas as spec, schemas as api};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};
pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list, upsert))
    .routes(utoipa_axum::routes!(remove))
}
fn view(mut row: Value) -> Value {
  let masked = num(&row, "masked") != 0;
  row["masked"] = json!(masked);
  row["protected"] = json!(num(&row, "protected") != 0);
  if masked {
    row["value"] = json!("");
  }
  row
}
#[utoipa::path(
    get,path="/projects/{id}/ci/variables",tag="variables",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Variable>>),api::Errors),
    security(("bearer"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  crate::access::project(&s, &h, p, 40).await?;
  reply(json!(
    s.repository_access
      .variables(p)
      .await?
      .into_iter()
      .map(gity_persistence::model_value)
      .map(view)
      .collect::<Vec<_>>()
  ))
}
#[utoipa::path(
    patch,path="/projects/{id}/ci/variables",tag="variables",
    params(api::ProjectPath),
    request_body(content=spec::SetVariable),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Variable>),api::Errors),
    security(("bearer"=[]))
)]
async fn upsert(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  crate::access::project(&s, &h, p, 40).await?;
  let key = required(&b, "key")?.to_uppercase();
  let value = text(&b, "value");
  let command = gity_projects::VariableCommand {
    key,
    value: value.into(),
    masked: b["masked"] == true,
    protected: b["protected"] == true,
  };
  let row = gity_persistence::model_value(s.repository_access.save_variable(p, command).await?);
  reply(view(row))
}
#[utoipa::path(
    delete,path="/projects/{id}/ci/variables/{key}",tag="variables",
    params(api::VariablePath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<api::Status>),api::Errors),
    security(("bearer"=[]))
)]
async fn remove(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, key)): Path<(i64, String)>,
) -> Reply {
  crate::access::project(&s, &h, p, 40).await?;
  s.repository_access
    .delete_variable(p, &key.trim().to_uppercase())
    .await?;
  reply(json!({"status":"deleted"}))
}
