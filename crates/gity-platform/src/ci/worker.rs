use super::*;
#[cfg(test)]
pub(crate) const ZERO: &str = gity_ci::ZERO;
pub(crate) const BLOCKED: &str = gity_ci::BLOCKED;
pub async fn worker_tick(s: &AppState) -> Result<(), AppError> {
  s.ci.worker_tick().await
}
pub async fn refresh_pipeline(s: &AppState, pipeline_id: i64) -> Result<(), AppError> {
  s.ci.refresh_pipeline(pipeline_id).await
}
pub(crate) async fn refresh_job_pipeline(s: &AppState, job: i64) -> Result<(), AppError> {
  s.ci.refresh_job_pipeline(job).await
}
