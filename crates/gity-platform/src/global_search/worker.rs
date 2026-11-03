use crate::AppState;
use gity_core::AppError;
use std::time::Duration;

/// Reconcile persisted database state and Git refs after restart or missed events.
pub async fn reconcile(s: &AppState) -> Result<(), AppError> {
  if !s.settings.search.global.enabled {
    return Ok(());
  }
  let mut after = 0;
  loop {
    let rows = s.projects.list_after(after, 100).await?;
    if rows.is_empty() {
      break;
    }
    for row in rows {
      after = row.id;
      s.global_search.jobs.mark_dirty(row.id).await?;
    }
  }
  Ok(())
}
pub async fn run_pending(s: &AppState) -> Result<(), AppError> {
  if !s.settings.search.global.enabled {
    return Ok(());
  }
  for _ in 0..4 {
    let Some(claim) = s.global_search.jobs.claim().await? else {
      break;
    };
    let result = tokio::time::timeout(Duration::from_secs(120), build(s, &claim)).await;
    match result {
      Ok(Ok(())) => {
        metrics::counter!("gity_global_search_jobs_total","result"=>"success").increment(1);
      }
      _ => {
        // Incomplete namespaces are never published. Clean them before allowing
        // a new claim where possible; a failed engine may defer cleanup.
        let _ = tokio::time::timeout(
          Duration::from_secs(5),
          s.global_search
            .engine()?
            .delete_namespace(claim.project_id, &claim.token),
        )
        .await;
        s.global_search.jobs.retry(&claim).await?;
        metrics::counter!("gity_global_search_jobs_total","result"=>"failure").increment(1);
        tracing::warn!(
          project_id = claim.project_id,
          generation = claim.generation,
          "global search rebuild deferred; retry scheduled"
        );
        // A failed project cannot spin in this tick or consume all worker time.
        break;
      }
    }
  }
  Ok(())
}
async fn build(s: &AppState, claim: &gity_persistence::search_jobs::Claim) -> Result<(), AppError> {
  let project = s.identity.active_project(claim.project_id).await?;
  let docs = if let Some(project) = project {
    super::projection::build(s, &project).await?
  } else {
    Vec::new()
  };
  if !s.global_search.jobs.is_current(claim).await? {
    return Ok(());
  }
  let prior = s.global_search.jobs.published(&[claim.project_id]).await?;
  s.global_search
    .engine()?
    .replace_project(claim.project_id, &claim.token, docs)
    .await
    .map_err(|_| AppError::internal("global search publication failed"))?;
  if s.global_search.jobs.complete(claim).await? {
    if let Some(old) = prior.get(&claim.project_id) {
      if let Err(error) = s
        .global_search
        .engine()?
        .delete_namespace(claim.project_id, old)
        .await
      {
        tracing::warn!(error=%error,project_id=claim.project_id,"obsolete search namespace cleanup deferred");
      }
    }
  } else {
    let _ = s
      .global_search
      .engine()?
      .delete_namespace(claim.project_id, &claim.token)
      .await;
  }
  Ok(())
}
pub async fn run(s: AppState) -> anyhow::Result<()> {
  if !s.settings.search.global.enabled {
    std::future::pending::<()>().await;
    return Ok(());
  }
  let mut interval = tokio::time::interval(Duration::from_secs(2));
  interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
  let mut reconcile_at = tokio::time::Instant::now();
  loop {
    interval.tick().await;
    if tokio::time::Instant::now() >= reconcile_at {
      if let Err(error) = reconcile(&s).await {
        tracing::warn!(error=%error,"global search reconciliation deferred");
      }
      reconcile_at = tokio::time::Instant::now()
        + Duration::from_secs(s.settings.search.refresh_interval_seconds.max(10));
    }
    if let Err(error) = run_pending(&s).await {
      tracing::warn!(error=%error,"global search worker iteration failed");
    }
  }
}
