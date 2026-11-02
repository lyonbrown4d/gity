use crate::{AppState, projects, repository};
use gity_core::AppError;

pub async fn refresh_indexes(state: &AppState) -> Result<(), AppError> {
  if !state.settings.search.index_enabled {
    return Ok(());
  }
  let mut cursor = 0i64;
  loop {
    let rows = state
      .projects
      .list_after(cursor, 100)
      .await?
      .into_iter()
      .map(gity_persistence::model_value)
      .collect::<Vec<_>>();
    if rows.is_empty() {
      break;
    }
    for project in rows {
      let id = projects::id(&project["id"])?;
      cursor = id;
      let result = if !project["deleted_at"].is_null() {
        state.search.delete(id).await
      } else {
        state
          .search
          .refresh(
            id,
            &repository::repo(&project),
            repository::default_branch(&project),
          )
          .await
          .map(|_| ())
      };
      if let Err(error) = result {
        metrics::counter!("gity_search_refresh_total","result"=>"failure").increment(1);
        tracing::warn!(error=%error,project_id=id,"periodic search index refresh failed");
      } else {
        metrics::counter!("gity_search_refresh_total","result"=>"success").increment(1);
      }
    }
  }
  Ok(())
}
pub async fn run(state: AppState) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(std::time::Duration::from_secs(
    state.settings.search.refresh_interval_seconds.max(1),
  ));
  interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
  loop {
    interval.tick().await;
    if let Err(error) = crate::features::recover_merge_operations(&state).await {
      tracing::error!(error=%error, "merge operation recovery scan failed");
    }
    #[cfg(feature = "packages")]
    if let Err(error) = state.artifacts.cleanup_pending(100).await {
      tracing::error!(error=%error, "artifact storage cleanup failed");
    }
    if state.settings.search.index_enabled
      && let Err(error) = refresh_indexes(&state).await
    {
      tracing::error!(error=%error,"search maintenance database scan failed");
    }
  }
}
