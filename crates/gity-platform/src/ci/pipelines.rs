use super::service::CreatePipelineInput;
use super::views::{pipeline_detail, pipeline_jobs};
use super::*;
use super::{params, pipeline_schemas as pipelines_spec};
use crate::access;
use crate::openapi::schemas as api;
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(list, create))
    .routes(utoipa_axum::routes!(lint))
    .routes(utoipa_axum::routes!(detail))
    .routes(utoipa_axum::routes!(list_jobs))
    .routes(utoipa_axum::routes!(refresh))
    .routes(utoipa_axum::routes!(cancel))
    .routes(utoipa_axum::routes!(retry))
}
#[utoipa::path(
    get, path="/{project}/pipelines", tag="ci",
    summary="List the latest 200 project pipelines",
    params(params::ProjectPath),
    responses((status=200,description="List the latest 200 project pipelines",body=api::Envelope<Vec<pipelines_spec::Pipeline>>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  access::project_action(&s, &h, p, ProjectAction::JobRead).await?;
  Ok(gity_core::ok(
    s.ci
      .pipelines(p)
      .await?
      .into_iter()
      .map(model_value)
      .collect::<Vec<_>>(),
  ))
}
#[utoipa::path(
    post, path="/{project}/pipelines", tag="ci",
    summary="Create a pipeline pinned to a Git commit",
    params(params::ProjectPath),
    request_body=CreatePipelineInput,
    responses((status=200,description="Create a pipeline pinned to a Git commit",body=api::Envelope<pipelines_spec::Detail>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn create(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<CreatePipelineInput>,
) -> Reply {
  let project = access::project_action(&s, &h, p, ProjectAction::JobWrite)
    .await?
    .project;
  if b.source == "push" {
    return Err(AppError::bad_request(
      "push pipelines are created by accepted Git updates",
    ));
  }
  Ok(gity_core::ok(create_pipeline(&s, &project, b).await?))
}
#[utoipa::path(
    post, path="/{project}/ci/lint", tag="ci",
    summary="Validate typed .gity-ci.yml without creating jobs",
    params(params::ProjectPath),
    request_body=pipelines_spec::Lint,
    responses((status=200,description="Validate typed .gity-ci.yml without creating jobs",body=api::Envelope<pipelines_spec::LintResult>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn lint(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  access::project_action(&s, &h, p, ProjectAction::JobWrite).await?;
  match config::parse(text(&b, "config_content")) {
    Ok(spec) => Ok(gity_core::ok(json!({"valid":true,"errors":[],"spec":spec}))),
    Err(e) => Ok(gity_core::ok(json!({"valid":false,"errors":[e.message]}))),
  }
}
#[utoipa::path(
    get, path="/{project}/pipelines/{pipeline}", tag="ci",
    summary="Get a pipeline configuration and job states",
    params(params::PipelinePath),
    responses((status=200,description="Get a pipeline configuration and job states",body=api::Envelope<pipelines_spec::Detail>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn detail(State(s): State<AppState>, h: HeaderMap, Path((p, i)): Path<(i64, i64)>) -> Reply {
  access::project_action(&s, &h, p, ProjectAction::JobRead).await?;
  Ok(gity_core::ok(pipeline_detail(&s, p, i).await?))
}
#[utoipa::path(
    get, path="/{project}/pipelines/{pipeline}/jobs", tag="ci",
    summary="List pipeline jobs in dependency order",
    params(params::PipelinePath),
    responses((status=200,description="List pipeline jobs in dependency order",body=api::Envelope<Vec<pipelines_spec::PipelineJob>>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn list_jobs(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, i)): Path<(i64, i64)>,
) -> Reply {
  access::project_action(&s, &h, p, ProjectAction::JobRead).await?;
  pipeline_row(&s, p, i).await?;
  Ok(gity_core::ok(pipeline_jobs(&s, p, i).await?))
}
#[utoipa::path(
    post, path="/{project}/pipelines/{pipeline}/refresh", tag="ci",
    summary="Reconcile pipeline state and release ready jobs",
    params(params::PipelinePath),
    responses((status=200,description="Reconcile pipeline state and release ready jobs",body=api::Envelope<pipelines_spec::Detail>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn refresh(State(s): State<AppState>, h: HeaderMap, Path((p, i)): Path<(i64, i64)>) -> Reply {
  access::project_action(&s, &h, p, ProjectAction::JobWrite).await?;
  pipeline_row(&s, p, i).await?;
  refresh_pipeline(&s, i).await?;
  Ok(gity_core::ok(pipeline_detail(&s, p, i).await?))
}
#[utoipa::path(
    post, path="/{project}/pipelines/{pipeline}/cancel", tag="ci",
    summary="Cancel pending and running pipeline jobs",
    params(params::PipelinePath),
    responses((status=200,description="Cancel pending and running pipeline jobs",body=api::Envelope<pipelines_spec::Detail>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn cancel(State(s): State<AppState>, h: HeaderMap, Path((p, i)): Path<(i64, i64)>) -> Reply {
  access::project_action(&s, &h, p, ProjectAction::JobWrite).await?;
  pipeline_row(&s, p, i).await?;
  s.ci.cancel_pipeline(p, i).await?;
  Ok(gity_core::ok(pipeline_detail(&s, p, i).await?))
}
#[utoipa::path(
    post, path="/{project}/pipelines/{pipeline}/retry", tag="ci",
    summary="Retry a terminal pipeline with new job attempts",
    params(params::PipelinePath),
    responses((status=200,description="Retry a terminal pipeline with new job attempts",body=api::Envelope<pipelines_spec::Detail>),api::Errors),
    security(("bearer"=[]),("private_token"=[]))
)]
async fn retry(State(s): State<AppState>, h: HeaderMap, Path((p, i)): Path<(i64, i64)>) -> Reply {
  access::project_action(&s, &h, p, ProjectAction::JobWrite).await?;
  s.ci.retry_pipeline(p, i).await?;
  Ok(gity_core::ok(pipeline_detail(&s, p, i).await?))
}
