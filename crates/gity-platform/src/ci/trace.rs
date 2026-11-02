use super::runners::owned;
use super::*;
use super::{params, runner_schemas as runners_spec};
use crate::openapi::schemas as api;
use axum::extract::{Path, State};
use base64::{Engine, engine::general_purpose::STANDARD};
#[utoipa::path(
    post, path="/runners/jobs/{job}/trace", tag="ci-runner-agent",
    summary="Append bounded trace output with streaming secret masking",
    params(params::AgentJobPath),
    request_body=runners_spec::AppendTrace,
    responses((status=200,description="Append bounded trace output with streaming secret masking",body=api::Envelope<runners_spec::Appended>),api::Errors),
    security(())
)]
pub(super) async fn append(
  State(s): State<AppState>,
  Path(j): Path<i64>,
  headers: axum::http::HeaderMap,
  Json(b): Json<Value>,
) -> Reply {
  let (_, job) = owned(&s, &b, j).await?;
  if text(&b, "output").len() > 1024 * 1024 {
    return Err(AppError::bad_request("trace chunk exceeds 1 MiB"));
  }
  let redaction_job = super::package_identity::redaction_job(&s, &headers, job.clone()).await?;
  let data = payload(&redaction_job)?;
  s.ci
    .append_trace(gity_ci::AppendTrace {
      lease: lease(&job),
      output: text(&b, "output").into(),
      masked_values: data["masked_values"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect(),
      final_chunk: b["final"] == true,
      truncated: b["output_truncated"] == true,
      duration_millis: b["duration_millis"].as_i64().unwrap_or(0),
    })
    .await?;
  Ok(gity_core::ok(json!({"appended":true})))
}
#[utoipa::path(
    post, path="/runners/jobs/{job}/source-archive", tag="ci-runner-agent",
    summary="Download the pinned job source as a base64 ZIP archive",
    params(params::AgentJobPath),
    request_body=runners_spec::Attempt,
    responses((status=200,description="Download the pinned job source as a base64 ZIP archive",body=api::Envelope<runners_spec::SourceArchive>),api::Errors),
    security(())
)]
pub(super) async fn source(
  State(s): State<AppState>,
  Path(j): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  let (_, job) = owned(&s, &b, j).await?;
  let data = payload(&job)?;
  let project = model_value(s.ci.project(job["project_id"].as_i64().unwrap()).await?);
  let bytes = s
    .git
    .archive(
      &crate::repository::repo(&project),
      text(&data, "commit_sha"),
    )
    .await
    .map_err(git_error)?;
  Ok(gity_core::ok(
    json!({"file_name":format!("source-{}.zip",text(&data,"commit_sha")),"encoding":"base64","content_base64":STANDARD.encode(bytes)}),
  ))
}

#[utoipa::path(
    post, path="/runners/jobs/{job}/source", tag="ci-runner-agent",
    summary="Stream the pinned job source ZIP with cancellation",
    params(params::AgentJobPath),
    request_body=runners_spec::Attempt,
    responses((status=200,description="Stream the pinned job source ZIP with cancellation",body=api::Binary,content_type="application/zip"),api::Errors),
    security(())
)]
pub(super) async fn source_stream(
  State(s): State<AppState>,
  Path(j): Path<i64>,
  Json(b): Json<Value>,
) -> Result<axum::response::Response, AppError> {
  let (_, job) = owned(&s, &b, j).await?;
  let data = payload(&job)?;
  let project = model_value(s.ci.project(job["project_id"].as_i64().unwrap()).await?);
  let process = s
    .git
    .spawn_archive(
      &crate::repository::repo(&project),
      text(&data, "commit_sha"),
    )
    .await
    .map_err(git_error)?;
  let reader = process.into_reader().map_err(git_error)?;
  use axum::response::IntoResponse;
  Ok(
    (
      [(axum::http::header::CONTENT_TYPE, "application/zip")],
      axum::body::Body::from_stream(tokio_util::io::ReaderStream::new(reader)),
    )
      .into_response(),
  )
}
