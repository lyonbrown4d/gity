use super::*;
use std::collections::BTreeMap;
// Keep internal sentinels inside the legacy MySQL TIMESTAMP range.
pub(crate) const ZERO: &str = "1970-01-02T00:00:00.000Z";
pub(crate) const BLOCKED: &str = "2037-12-31T00:00:00.000Z";

/// Renewed leases use the same timestamp format as comparisons in every dialect.
pub async fn worker_tick(s: &AppState) -> Result<(), AppError> {
  let timestamp = now();
  s.db.execute("UPDATE project_jobs SET status='succeeded',attempts=attempts+1,result='noop completed',started_at=?,finished_at=?,updated_at=? WHERE kind='noop' AND status='pending' AND run_after<=?",
        vec![timestamp.clone().into(),
            timestamp.clone().into(),
            timestamp.clone().into(),
            timestamp.clone().into()]).await?;
  s.db.execute("UPDATE project_jobs SET status=CASE WHEN attempts>=max_attempts THEN 'failed' ELSE 'pending' END,last_error='runner lease expired',locked_by='',locked_until=?,run_after=?,updated_at=? WHERE kind='script' AND status='running' AND locked_until<?",
        vec![timestamp.clone().into(),
            timestamp.clone().into(),
            timestamp.clone().into(),
            timestamp.clone().into()]).await?;
  s.db
    .execute(
      "UPDATE project_runners SET status='offline' WHERE last_contact_at<?",
      vec![
        (chrono::Utc::now() - chrono::Duration::minutes(2))
          .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
          .into(),
      ],
    )
    .await?;
  let pipelines=s.db.all("SELECT id FROM project_pipelines WHERE status IN ('pending','running') ORDER BY created_at LIMIT 200",vec![]).await?;
  for pipeline in pipelines {
    refresh_pipeline(s, id(&pipeline)).await?;
  }
  Ok(())
}

/// Lock the pipeline row before reconciling its jobs so cancellation, completion,
/// and dependency release cannot overwrite each other's terminal state.
pub async fn refresh_pipeline(s: &AppState, pipeline_id: i64) -> Result<(), AppError> {
  let tx = s.db.transaction().await?;
  tx.execute_raw(s.db.statement(
    "UPDATE project_pipelines SET updated_at=? WHERE id=?",
    vec![now().into(), pipeline_id.into()],
  ))
  .await?;
  let pipeline = tx_rows(
    s,
    &tx,
    "SELECT * FROM project_pipelines WHERE id=?",
    vec![pipeline_id.into()],
  )
  .await?;
  let row = pipeline
    .first()
    .ok_or_else(|| AppError::not_found("pipeline not found"))?;
  if ["canceled", "succeeded", "failed"].contains(&text(row, "status")) {
    tx.commit().await?;
    return Ok(());
  }
  let jobs=tx_rows(s,
        &tx,
        "SELECT j.*,l.name AS ci_name,l.needs AS ci_needs FROM project_jobs j JOIN project_pipeline_jobs l ON j.id=l.project_job_id WHERE l.pipeline_id=?",
        vec![pipeline_id.into()]).await?;
  let states = jobs
    .iter()
    .map(|j| (text(j, "ci_name"), text(j, "status")))
    .collect::<BTreeMap<_, _>>();
  let failed = jobs.iter().any(|j| j["status"] == "failed");
  let canceled = jobs.iter().any(|j| j["status"] == "canceled");
  let succeeded = !jobs.is_empty() && jobs.iter().all(|j| j["status"] == "succeeded");
  let running = jobs
    .iter()
    .any(|j| j["status"] == "running" || j["status"] == "succeeded");
  let status = if failed {
    "failed"
  } else if canceled {
    "canceled"
  } else if succeeded {
    "succeeded"
  } else if running {
    "running"
  } else {
    "pending"
  };
  for job in &jobs {
    if ["failed", "canceled"].contains(&status)
      && ["pending", "running"].contains(&text(job, "status"))
    {
      tx.execute_raw(s.db.statement("UPDATE project_jobs SET status='canceled',finished_at=?,updated_at=?,locked_by='',locked_until=? WHERE id=? AND status IN ('pending','running')",
                    vec![now().into(),
                        now().into(),
                        now().into(),
                        id(job).into()])).await?;
    } else if status != "succeeded"
      && text(job, "status") == "pending"
      && text(job, "run_after").starts_with(&BLOCKED[..10])
    {
      let needs: Vec<String> = serde_json::from_str(text(job, "ci_needs"))
        .map_err(|_| AppError::internal("invalid pipeline dependencies"))?;
      if needs
        .iter()
        .all(|need| states.get(need.as_str()) == Some(&"succeeded"))
      {
        tx.execute_raw(s.db.statement("UPDATE project_jobs SET run_after=?,updated_at=? WHERE id=? AND status='pending' AND run_after=?",
                        vec![now().into(),
                            now().into(),
                            id(job).into(),
                            BLOCKED.into()])).await?;
      }
    }
  }
  // PostgreSQL cannot infer a timestamp from this CASE result placeholder.
  // SQLite keeps the repository's canonical RFC3339 text representation.
  let started_at: SqlValue = match s.db.backend() {
    sea_orm::DatabaseBackend::Sqlite => now().into(),
    _ => chrono::Utc::now().naive_utc().into(),
  };
  tx.execute_raw(s.db.statement("UPDATE project_pipelines SET status=?,started_at=CASE WHEN started_at=? AND ?<>'pending' THEN ? ELSE started_at END,finished_at=?,updated_at=? WHERE id=? AND status NOT IN ('canceled','failed','succeeded')",
            vec![status.into(),
                ZERO.into(),
                status.into(),
                started_at,
                if ["succeeded",
                    "failed",
                    "canceled"].contains(&status){
                    now().into()
                }
                else{
                    ZERO.into()
                }
                ,
                now().into(),
                pipeline_id.into()])).await?;
  tx.commit().await?;
  Ok(())
}
pub(crate) async fn refresh_job_pipeline(s: &AppState, job: i64) -> Result<(), AppError> {
  if let Some(link) = s
    .db
    .one(
      "SELECT pipeline_id FROM project_pipeline_jobs WHERE project_job_id=?",
      vec![job.into()],
    )
    .await?
  {
    refresh_pipeline(
      s,
      link["pipeline_id"]
        .as_i64()
        .ok_or_else(|| AppError::internal("invalid pipeline id"))?,
    )
    .await?;
  }
  Ok(())
}
