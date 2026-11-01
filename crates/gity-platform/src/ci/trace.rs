use super::claims::redact;
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
  let attempt = job["attempts"].as_i64().unwrap();
  let project = job["project_id"].as_i64().unwrap();
  let tx = s.db.transaction().await?;
  let changed=tx.execute_raw(s.db.statement("UPDATE project_jobs SET updated_at=? WHERE id=? AND status='running' AND attempts=? AND locked_by=? AND locked_until>?",
            vec![now().into(),
                j.into(),
                attempt.into(),
                text(&job,
                    "locked_by").into(),
                now().into()])).await?.rows_affected();
  if changed != 1 {
    return Err(AppError::conflict("job lease changed"));
  }
  let current = tx_rows(
    &s,
    &tx,
    "SELECT * FROM project_jobs WHERE id=?",
    vec![j.into()],
  )
  .await?;
  let mut data = payload(&current[0])?;
  let mut masking = data.clone();
  masking["masked_values"] = payload(&redaction_job)?["masked_values"].clone();
  let (output, pending) = redact_chunk(&masking, text(&b, "output"), b["final"] == true)?;
  data["trace_pending"] = json!(pending);
  tx.execute_raw(s.db.statement(
    "UPDATE project_jobs SET payload=? WHERE id=?",
    vec![data.to_string().into(), j.into()],
  ))
  .await?;
  let existing = tx_rows(
    &s,
    &tx,
    "SELECT * FROM project_job_logs WHERE project_job_id=? AND attempt=?",
    vec![j.into(), attempt.into()],
  )
  .await?;
  let max = 4 * 1024 * 1024;
  let duration = b["duration_millis"].as_i64().unwrap_or(0).max(0);
  if let Some(log) = existing.first() {
    let mut combined = format!("{}{}", text(log, "output"), output);
    let truncated =
      combined.len() > max || b["output_truncated"] == true || flag(log, "output_truncated");
    if combined.len() > max {
      let mut end = max;
      while !combined.is_char_boundary(end) {
        end -= 1;
      }
      combined.truncate(end);
    }
    tx.execute_raw(s.db.statement("UPDATE project_job_logs SET output=?,output_truncated=?,duration_millis=?,updated_at=? WHERE id=?",
                vec![combined.into(),
                    (truncated as i64).into(),
                    duration.into(),
                    now().into(),
                    id(log).into()])).await?;
  } else {
    insert_tx(
      &s,
      &tx,
      "project_job_logs",
      json!({
                  "project_id":project,
                  "project_job_id":j,
                  "attempt":attempt,
                  "exit_code":0,
                  "output":output,
                  "output_truncated":i64::from(b["output_truncated"]==true),
                  "duration_millis":duration
              }
      ),
    )
    .await?;
  }
  tx.commit().await?;
  Ok(gity_core::ok(json!({"appended":true})))
}
// Hold a suffix that may be the beginning of a masked secret until a following
// chunk proves whether it matches. GET trace never exposes a split secret.
fn redact_chunk(
  data: &Value,
  input: &str,
  final_chunk: bool,
) -> Result<(String, String), AppError> {
  let combined = format!("{}{}", text(data, "trace_pending"), input);
  let job = json!({"payload":data.to_string()});
  let mut output = redact(&job, &combined)?;
  let mut retained = 0;
  if let Some(masked) = data["masked_values"].as_array() {
    for secret in masked.iter().filter_map(Value::as_str) {
      for (index, _) in secret.char_indices().skip(1) {
        if output.ends_with(&secret[..index]) {
          retained = retained.max(index);
        }
      }
    }
  }
  if retained == 0 {
    return Ok((output, String::new()));
  }
  let pending = output.split_off(output.len() - retained);
  if final_chunk {
    output.push_str("[MASKED]");
    Ok((output, String::new()))
  } else {
    Ok((output, pending))
  }
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
  let project = s
    .db
    .one(
      "SELECT * FROM projects WHERE id=? AND deleted_at IS NULL",
      vec![job["project_id"].as_i64().unwrap().into()],
    )
    .await?
    .ok_or_else(|| AppError::not_found("project not found"))?;
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
  let project = s
    .db
    .one(
      "SELECT * FROM projects WHERE id=? AND deleted_at IS NULL",
      vec![job["project_id"].as_i64().unwrap().into()],
    )
    .await?
    .ok_or_else(|| AppError::not_found("project not found"))?;
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
