use super::*;
use super::{job_schemas as jobs_spec, params};
use crate::access;
use crate::openapi::schemas as api;
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};
pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list, create))
    .routes(utoipa_axum::routes!(detail))
    .routes(utoipa_axum::routes!(cancel))
    .routes(utoipa_axum::routes!(retry))
    .routes(utoipa_axum::routes!(trace))
    .routes(utoipa_axum::routes!(super::artifacts::list))
    .routes(utoipa_axum::routes!(super::artifacts::download))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    post, path="/{project}/jobs", tag="ci",
    summary="Enqueue a script or noop project job",
    params(params::ProjectPath),
    request_body=jobs_spec::CreateJob,
    responses((status=200,description="Enqueue a script or noop project job",body=api::Envelope<jobs_spec::Job>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn create(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  let project = access::project_action(&s, &h, p, ProjectAction::JobWrite)
    .await?
    .project;
  let kind = text(&b, "kind");
  if !["noop", "script"].contains(&kind) {
    return Err(AppError::bad_request(
      "supported job kinds are noop and script",
    ));
  }
  let attempts = b["max_attempts"].as_i64().unwrap_or(3);
  if !(1..=10).contains(&attempts) {
    return Err(AppError::bad_request("max_attempts must be 1–10"));
  }
  let mut body = text(&b, "payload").to_owned();
  if kind == "script" {
    let mut data: Value = serde_json::from_str(&body)
      .map_err(|_| AppError::bad_request("script job payload must be JSON"))?;
    let mut job = json!({"script":data["script"]});
    for key in ["image", "tags", "artifacts", "timeout_seconds"] {
      if let Some(value) = data.get(key) {
        job[key] = value.clone();
      }
    }
    if let Some(env) = data.get("env") {
      job["variables"] = env.clone();
    }
    let spec = config::parse(&json!({"jobs":{"manual":job}}).to_string())?;
    let validated = &spec.jobs["manual"];
    data["script"] = json!(validated.script.lines());
    data["tags"] = json!(validated.tags);
    data["artifacts"] = json!(validated.artifacts.paths());
    data["image"] = json!(validated.image);
    data["timeout_seconds"] = json!(validated.timeout_seconds);
    data["yaml_env"] = json!(validated.variables);
    data["env"] = json!(validated.variables);
    data["project_full_path"] = project["full_path"].clone();
    data["pipeline_job_name"] = json!("manual");
    data["trusted_config"] = json!(false);
    if !text(&data, "commit_sha").is_empty() {
      data["commit_sha"] = json!(
        s.git
          .resolve_revision(
            &crate::repository::repo(&project),
            text(&data, "commit_sha"),
            ""
          )
          .await
          .map_err(git_error)?
      );
    }
    body = data.to_string();
  }
  let run_after = if text(&b, "run_after").is_empty() {
    now()
  } else {
    chrono::DateTime::parse_from_rfc3339(text(&b, "run_after"))
      .map_err(|_| AppError::bad_request("run_after must be RFC3339"))?
      .with_timezone(&chrono::Utc)
      .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
  };
  let row = model_value(
    s.ci
      .create_job(gity_ci::NewJob {
        project_id: p,
        kind: kind.into(),
        payload: body,
        max_attempts: attempts as i32,
        run_after: chrono::DateTime::parse_from_rfc3339(&run_after)
          .map_err(|_| AppError::bad_request("invalid run_after"))?
          .naive_utc(),
      })
      .await?,
  );
  Ok(gity_core::ok(public_job(row)))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get, path="/{project}/jobs", tag="ci",
    summary="List the latest 200 project jobs without secrets",
    params(params::ProjectPath),
    responses((status=200,description="List the latest 200 project jobs without secrets",body=api::Envelope<Vec<jobs_spec::Job>>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  access::project_action(&s, &h, p, ProjectAction::JobRead).await?;
  Ok(gity_core::ok(
    s.ci
      .jobs(p)
      .await?
      .into_iter()
      .map(model_value)
      .map(public_job)
      .collect::<Vec<_>>(),
  ))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get, path="/{project}/jobs/{job}", tag="ci",
    summary="Get a project job without environment secrets",
    params(params::JobPath),
    responses((status=200,description="Get a project job without environment secrets",body=api::Envelope<jobs_spec::Job>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn detail(State(s): State<AppState>, h: HeaderMap, Path((p, j)): Path<(i64, i64)>) -> Reply {
  access::project_action(&s, &h, p, ProjectAction::JobRead).await?;
  Ok(gity_core::ok(public_job(job_row(&s, p, j).await?)))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    post, path="/{project}/jobs/{job}/cancel", tag="ci",
    summary="Cancel a live project job and revoke its lease",
    params(params::JobPath),
    responses((status=200,description="Cancel a live project job and revoke its lease",body=api::Envelope<jobs_spec::Job>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn cancel(State(s): State<AppState>, h: HeaderMap, Path((p, j)): Path<(i64, i64)>) -> Reply {
  access::project_action(&s, &h, p, ProjectAction::JobWrite).await?;
  job_row(&s, p, j).await?;
  s.ci.cancel_job(p, j).await?;
  Ok(gity_core::ok(public_job(job_row(&s, p, j).await?)))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    post, path="/{project}/jobs/{job}/retry", tag="ci",
    summary="Retry a failed or canceled job and release dependents",
    params(params::JobPath),
    responses((status=200,description="Retry a failed or canceled job and release dependents",body=api::Envelope<jobs_spec::Job>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn retry(State(s): State<AppState>, h: HeaderMap, Path((p, j)): Path<(i64, i64)>) -> Reply {
  access::project_action(&s, &h, p, ProjectAction::JobWrite).await?;
  s.ci.retry_job(p, j).await?;
  Ok(gity_core::ok(public_job(job_row(&s, p, j).await?)))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get, path="/{project}/jobs/{job}/trace", tag="ci",
    summary="Get masked job trace output and attempt history",
    params(params::JobPath),
    responses((status=200,description="Get masked job trace output and attempt history",body=api::Envelope<jobs_spec::Trace>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn trace(State(s): State<AppState>, h: HeaderMap, Path((p, j)): Path<(i64, i64)>) -> Reply {
  access::project_action(&s, &h, p, ProjectAction::JobRead).await?;
  let job = job_row(&s, p, j).await?;
  let logs = s
    .ci
    .logs(p, j)
    .await?
    .into_iter()
    .map(model_value)
    .collect::<Vec<_>>();
  let latest = logs
    .last()
    .cloned()
    .unwrap_or(json!({"output":"","exit_code":0,"attempt":job["attempts"]}));
  Ok(gity_core::ok(
    json!({"job":public_job(job),"attempts":logs,"log":latest,"output":latest["output"]}),
  ))
}
