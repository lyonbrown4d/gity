use super::*;
use super::{params, runner_schemas as runners_spec};
use crate::openapi::schemas as api;
use crate::{access, auth};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};

pub(super) fn project_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list, register))
    .routes(utoipa_axum::routes!(remove))
}
pub(super) fn agent_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  let (schemas, paths, upload) = utoipa_axum::routes!(super::artifacts::upload);
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(heartbeat))
    .routes(utoipa_axum::routes!(super::claims::claim))
    .routes(utoipa_axum::routes!(super::claims::complete))
    .routes(utoipa_axum::routes!(super::claims::fail))
    .routes(utoipa_axum::routes!(super::trace::append))
    .routes(utoipa_axum::routes!(super::trace::source))
    .routes(utoipa_axum::routes!(super::trace::source_stream))
    .routes((
      schemas,
      paths,
      upload.layer(axum::extract::DefaultBodyLimit::max(90 * 1024 * 1024)),
    ))
}
pub(super) fn public_runner(mut row: Value) -> Value {
  if let Some(map) = row.as_object_mut() {
    map.remove("token");
  }
  row["active"] = json!(flag(&row, "active"));
  row
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get, path="/{project}/runners", tag="ci",
    summary="List registered project runners without tokens",
    params(params::ProjectPath),
    responses((status=200,description="List registered project runners without tokens",body=api::Envelope<Vec<runners_spec::Runner>>),api::Errors),
    security(("bearer"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  access::project_action(&s, &h, p, ProjectAction::RunnerRead).await?;
  Ok(gity_core::ok(
    s.ci
      .runners(p)
      .await?
      .into_iter()
      .map(model_value)
      .map(public_runner)
      .collect::<Vec<_>>(),
  ))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    post, path="/{project}/runners", tag="ci",
    summary="Register a project runner and return its token once",
    params(params::ProjectPath),
    request_body=runners_spec::Register,
    responses((status=200,description="Register a project runner and return its token once",body=api::Envelope<runners_spec::Registration>),api::Errors),
    security(("bearer"=[]))
)]
async fn register(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  access::project(&s, &h, p, 40).await?;
  if text(&b, "name").trim().is_empty() {
    return Err(AppError::bad_request("runner name is required"));
  }
  let token = format!("grt_{}", auth::secret());
  let row = model_value(
    s.ci
      .register_runner(gity_ci::NewRunner {
        project_id: p,
        name: text(&b, "name").into(),
        description: text(&b, "description").into(),
        tags: text(&b, "tags").into(),
        token_hash: format!("sha256:{}", auth::token_hash(&token)),
      })
      .await?,
  );
  crate::standalone::runner::enroll(&s, id(&row), &token).await?;
  Ok(gity_core::ok(
    json!({"runner":public_runner(row),"token":token}),
  ))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    delete, path="/{project}/runners/{runner}", tag="ci",
    summary="Revoke and remove a project runner",
    params(params::RunnerPath),
    responses((status=200,description="Revoke and remove a project runner",body=api::Envelope<runners_spec::Runner>),api::Errors),
    security(("bearer"=[]))
)]
async fn remove(State(s): State<AppState>, h: HeaderMap, Path((p, r)): Path<(i64, i64)>) -> Reply {
  access::project(&s, &h, p, 40).await?;
  let row = model_value(s.ci.remove_runner(p, r).await?);
  crate::standalone::runner::revoke(&s, r).await?;
  Ok(gity_core::ok(public_runner(row)))
}
pub(crate) async fn authenticate(s: &AppState, b: &Value) -> Result<Value, AppError> {
  let token = text(b, "token");
  if token.is_empty() {
    return Err(AppError::unauthorized("runner token required"));
  }
  let row = model_value(
    s.ci
      .authenticate_runner(&format!("sha256:{}", auth::token_hash(token)), token)
      .await?,
  );
  Ok(row)
}
pub(crate) async fn owned(s: &AppState, b: &Value, j: i64) -> Result<(Value, Value), AppError> {
  let runner = authenticate(s, b).await?;
  let job = job_row(s, runner["project_id"].as_i64().unwrap(), j).await?;
  let attempt = b["attempt"].as_i64().unwrap_or(0);
  if text(&job, "status") != "running"
    || text(&job, "locked_by") != format!("runner:{}", id(&runner))
    || text(&job, "locked_until") <= now().as_str()
    || (attempt != job["attempts"].as_i64().unwrap_or(0) && (attempt != 0 || job["attempts"] != 1))
  {
    return Err(AppError::conflict(
      "runner does not own a live lease for this job attempt",
    ));
  }
  Ok((runner, job))
}
#[utoipa::path(
    post, path="/runners/heartbeat", tag="ci-runner-agent",
    summary="Renew live runner leases and return active job IDs",
    request_body=runners_spec::Token,
    responses((status=200,description="Renew live runner leases and return active job IDs",body=api::Envelope<runners_spec::Heartbeat>),api::Errors),
    security(())
)]
async fn heartbeat(State(s): State<AppState>, Json(b): Json<Value>) -> Reply {
  let runner = authenticate(&s, &b).await?;
  let rows = s
    .ci
    .heartbeat(runner["project_id"].as_i64().unwrap(), id(&runner))
    .await?
    .into_iter()
    .map(model_value)
    .collect::<Vec<_>>();
  let mut response = public_runner(runner);
  response["running_job_ids"] = json!(rows.iter().map(|r| id(r).to_string()).collect::<Vec<_>>());
  response["running_job_attempts"] = json!(
    rows
      .iter()
      .map(|row| (id(row).to_string(), row["attempts"].as_i64().unwrap_or(0)))
      .collect::<std::collections::BTreeMap<_, _>>()
  );
  Ok(gity_core::ok(response))
}
