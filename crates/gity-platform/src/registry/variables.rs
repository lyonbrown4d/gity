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
    s.db
      .all(
        &format!(
          "SELECT * FROM project_ci_variables WHERE project_id = ? ORDER BY {}",
          s.db.quote_identifier("key")?
        ),
        vec![p.into()]
      )
      .await?
      .into_iter()
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
  if key.len() > 128
    || !regex::Regex::new("^[A-Z_][A-Z0-9_]*$")
      .unwrap()
      .is_match(&key)
  {
    return Err(AppError::bad_request("ci variable key is invalid"));
  }
  let value = text(&b, "value");
  if value.len() > 64 * 1024 {
    return Err(AppError::bad_request("ci variable value is too large"));
  }
  if b["masked"] == true && (value.len() < 8 || value.contains(['\n', '\r'])) {
    return Err(AppError::bad_request(
      "masked ci variable value must be at least 8 characters",
    ));
  }
  let key_column = s.db.quote_identifier("key")?;
  let sql = format!("SELECT * FROM project_ci_variables WHERE project_id = ? AND {key_column} = ?");
  let args = vec![p.into(), key.clone().into()];
  let data = map(json!({
      "project_id":p,"key":key,"value":value,
      "masked":i64::from(b["masked"]==true),
      "protected":i64::from(b["protected"]==true)
  }));
  let row = if let Some(row) = s.db.one(&sql, args.clone()).await? {
    s.db
      .update("project_ci_variables", num(&row, "id"), data)
      .await?
  } else {
    match s.db.insert("project_ci_variables", data.clone()).await {
      Ok(row) => row,
      Err(error) if error.status == axum::http::StatusCode::CONFLICT => {
        let existing = s.db.one(&sql, args).await?.ok_or(error)?;
        s.db
          .update("project_ci_variables", num(&existing, "id"), data)
          .await?
      }
      Err(error) => return Err(error),
    }
  };
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
  let sql = format!(
    "SELECT id FROM project_ci_variables WHERE project_id = ? AND {} = ?",
    s.db.quote_identifier("key")?
  );
  if let Some(row) = s
    .db
    .one(&sql, vec![p.into(), key.trim().to_uppercase().into()])
    .await?
  {
    s.db.delete("project_ci_variables", num(&row, "id")).await?;
  }
  reply(json!({"status":"deleted"}))
}
