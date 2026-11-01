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
    let rows=s.db.all("SELECT * FROM project_jobs WHERE project_id=? AND kind='script' AND status='pending' AND run_after<=? AND attempts<max_attempts ORDER BY run_after,id LIMIT 100",
        vec![project.into(),
            now().into()]).await?;
    let tags = text(&runner, "tags")
        .split(',')
        .map(|v| v.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    for row in rows {
        let mut data = payload(&row)?;
        if data["tags"].as_array().is_some_and(|need| {
            need.iter()
                .any(|v| !tags.contains(&v.as_str().unwrap_or_default().to_ascii_lowercase()))
        }) {
            continue;
        }
        let lease = b["lease_seconds"].as_i64().unwrap_or(180).clamp(30, 3600);
        let until = (chrono::Utc::now() + chrono::Duration::seconds(lease))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let (env, masked) = variables::environment(&s, project, &data).await?;
        let tx = s.db.transaction().await?;
        let changed=tx.execute_raw(s.db.statement("UPDATE project_jobs SET status='running',attempts=attempts+1,locked_by=?,locked_until=?,started_at=?,updated_at=? WHERE id=? AND status='pending' AND attempts=? AND run_after<=?",
                vec![format!("runner:{}",
                        id(&runner)).into(),
                    until.into(),
                    now().into(),
                    now().into(),
                    id(&row).into(),
                    row["attempts"].as_i64().unwrap_or(0).into(),
                    now().into()])).await?.rows_affected();
        if changed == 0 {
            tx.rollback().await?;
            continue;
        }
        data["env"] = json!(env);
        data["masked_values"] = json!(masked);
        tx.execute_raw(s.db.statement(
            "UPDATE project_jobs SET payload=? WHERE id=?",
            vec![data.to_string().into(), id(&row).into()],
        ))
        .await?;
        tx.commit().await?;
        worker::refresh_job_pipeline(&s, id(&row)).await?;
        let job = one(&s, "project_jobs", project, id(&row)).await?;
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
    Json(b): Json<Value>,
) -> Reply {
    finish(&s, j, &b, true).await
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
    Json(b): Json<Value>,
) -> Reply {
    finish(&s, j, &b, false).await
}
async fn finish(s: &AppState, j: i64, b: &Value, success: bool) -> Reply {
    let (_, job) = owned(s, b, j).await?;
    let result = redact(&job, text(b, "result"))?;
    let error = redact(&job, text(b, "error"))?;
    let changed=s.db.execute("UPDATE project_jobs SET status=?,result=?,last_error=?,finished_at=?,updated_at=?,locked_by='',locked_until=? WHERE id=? AND status='running' AND locked_by=? AND attempts=? AND locked_until>?",
        vec![if success{
                "succeeded".into()
            }
            else{
                "failed".into()
            }
            ,
            result.into(),
            error.into(),
            now().into(),
            now().into(),
            now().into(),
            j.into(),
            text(&job,
                "locked_by").into(),
            job["attempts"].as_i64().unwrap_or(0).into(),
            now().into()]).await?;
    if changed != 1 {
        return Err(AppError::conflict("job lease changed"));
    }
    s.db.execute(
        "UPDATE project_job_logs SET exit_code=?,updated_at=? WHERE project_job_id=? AND attempt=?",
        vec![
            b["exit_code"]
                .as_i64()
                .unwrap_or(if success { 0 } else { 1 })
                .into(),
            now().into(),
            j.into(),
            job["attempts"].as_i64().unwrap_or(0).into(),
        ],
    )
    .await?;
    worker::refresh_job_pipeline(s, j).await?;
    Ok(gity_core::ok(public_job(
        one(s, "project_jobs", job["project_id"].as_i64().unwrap(), j).await?,
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
