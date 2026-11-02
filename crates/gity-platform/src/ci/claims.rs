use super::runners::{authenticate, owned, public_runner};
use super::*;
use super::{job_schemas as jobs_spec, params, runner_schemas as runners_spec};
use crate::openapi::schemas as api;
use axum::extract::{Path, State};
#[utoipa::path(
    post, path="/runners/jobs/claim", tag="ci-runner-agent",
    summary="Atomically claim a ready job matching runner tags",
    request_body=runners_spec::Claim,
    responses((status=200,description="Atomically claim a ready job matching runner tags",body=api::Envelope<runners_spec::Claimed>),api::Errors),
    security(())
)]
pub(super) async fn claim(State(s): State<AppState>, Json(b): Json<Value>) -> Reply {
  let runner = authenticate(&s, &b).await?;
  worker_tick(&s).await?;
  let project = runner["project_id"].as_i64().unwrap();
  let rows = s
    .ci
    .ready_jobs(project)
    .await?
    .into_iter()
    .map(model_value)
    .collect::<Vec<_>>();
  let tags = text(&runner, "tags")
    .split(',')
    .map(|v| v.trim().to_ascii_lowercase())
    .collect::<Vec<_>>();
  for row in rows {
    let mut data = payload(&row)?;
    if data["tags"].as_array().is_some_and(|need| {
      need
        .iter()
        .any(|v| !tags.contains(&v.as_str().unwrap_or_default().to_ascii_lowercase()))
    }) {
      continue;
    }
    let lease = b["lease_seconds"].as_i64().unwrap_or(180).clamp(30, 3600);
    let (env, masked) = variables::environment(&s, project, &data).await?;
    let credential = if b["job_package_token"] == true {
      let (raw, digest) =
        super::package_identity::issue(id(&row), row["attempts"].as_i64().unwrap_or(0) + 1);
      data[super::package_identity::HASH_FIELD] = json!(digest);
      Some(raw)
    } else {
      data
        .as_object_mut()
        .unwrap()
        .remove(super::package_identity::HASH_FIELD);
      None
    };
    data["env"] = json!(env);
    data["masked_values"] = json!(masked);
    if !s
      .ci
      .claim_job(gity_ci::ClaimJob {
        job_id: id(&row),
        attempt: row["attempts"].as_i64().unwrap_or(0) as i32,
        runner_id: id(&runner),
        lease_seconds: lease,
        payload: data.to_string(),
      })
      .await?
    {
      continue;
    }
    worker::refresh_job_pipeline(&s, id(&row)).await?;
    let mut job = job_row(&s, project, id(&row)).await?;
    let mut response_data = payload(&job)?;
    response_data
      .as_object_mut()
      .unwrap()
      .remove(super::package_identity::HASH_FIELD);
    if let Some(raw) = credential {
      response_data["env"]["CI_JOB_TOKEN"] = json!(raw);
      response_data["masked_values"]
        .as_array_mut()
        .unwrap()
        .push(json!(raw));
    }
    job["payload"] = json!(response_data.to_string());
    return Ok(gity_core::ok(
      json!({"claimed":true,"runner":public_runner(runner),"job":job}),
    ));
  }
  Ok(gity_core::ok(
    json!({"claimed":false,"runner":public_runner(runner)}),
  ))
}
#[utoipa::path(
    post, path="/runners/jobs/{job}/complete", tag="ci-runner-agent",
    summary="Complete a live job attempt owned by this runner",
    params(params::AgentJobPath),
    request_body=runners_spec::Finish,
    responses((status=200,description="Complete a live job attempt owned by this runner",body=api::Envelope<jobs_spec::Job>),api::Errors),
    security(())
)]
pub(super) async fn complete(
  State(s): State<AppState>,
  Path(j): Path<i64>,
  headers: axum::http::HeaderMap,
  Json(b): Json<Value>,
) -> Reply {
  finish(&s, &headers, j, &b, true).await
}
#[utoipa::path(
    post, path="/runners/jobs/{job}/fail", tag="ci-runner-agent",
    summary="Fail a live job attempt owned by this runner",
    params(params::AgentJobPath),
    request_body=runners_spec::Finish,
    responses((status=200,description="Fail a live job attempt owned by this runner",body=api::Envelope<jobs_spec::Job>),api::Errors),
    security(())
)]
pub(super) async fn fail(
  State(s): State<AppState>,
  Path(j): Path<i64>,
  headers: axum::http::HeaderMap,
  Json(b): Json<Value>,
) -> Reply {
  finish(&s, &headers, j, &b, false).await
}
async fn finish(
  s: &AppState,
  headers: &axum::http::HeaderMap,
  j: i64,
  b: &Value,
  success: bool,
) -> Reply {
  let (_, job) = owned(s, b, j).await?;
  let job = super::package_identity::redaction_job(s, headers, job).await?;
  let result = redact(&job, text(b, "result"))?;
  let error = redact(&job, text(b, "error"))?;
  s.ci
    .finish_job(gity_ci::FinishJob {
      lease: lease(&job),
      success,
      result,
      error,
      exit_code: b["exit_code"]
        .as_i64()
        .unwrap_or(if success { 0 } else { 1 }) as i32,
    })
    .await?;
  if success && text(&job, "kind") == crate::code_index::JOB_KIND {
    // Indexing is a follow-up concern: an ingest failure must not rewrite the
    // already accepted job completion.
    let project_id = job["project_id"].as_i64().unwrap_or_default();
    match crate::code_index::ingest(s, project_id, &job).await {
      Ok(()) => {
        metrics::counter!("gity_code_index_ingest_total", "result" => "success").increment(1);
      }
      Err(error) => {
        metrics::counter!("gity_code_index_ingest_total", "result" => "failure").increment(1);
        tracing::warn!(error=%error, job_id=j, project_id, "code index ingest failed");
      }
    }
  }
  Ok(gity_core::ok(public_job(
    job_row(s, job["project_id"].as_i64().unwrap(), j).await?,
  )))
}
pub(crate) fn redact(job: &Value, input: &str) -> Result<String, AppError> {
  let mut output = input.to_owned();
  if let Some(masked) = payload(job)?["masked_values"].as_array() {
    let mut secrets = masked
      .iter()
      .filter_map(Value::as_str)
      .filter(|v| !v.is_empty())
      .collect::<Vec<_>>();
    secrets.sort_by_key(|v| std::cmp::Reverse(v.len()));
    for value in secrets {
      output = output.replace(value, "[MASKED]");
    }
  }
  Ok(output)
}
