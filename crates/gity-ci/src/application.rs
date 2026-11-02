use crate::*;
use gity_core::AppError;
use gity_entities::{
  project_branch_protections, project_ci_variables as variable, project_job_artifacts as artifact,
  project_job_logs as log, project_jobs as job, project_pipeline_jobs as link,
  project_pipelines as pipeline, project_runners as runner, projects,
};
/// Application entrypoint shared by HTTP and worker adapters.
impl CiService {
  pub async fn project(&self, id: i64) -> Result<projects::Model, AppError> {
    self.repository.project(id).await
  }
  pub async fn protections(
    &self,
    p: i64,
  ) -> Result<Vec<project_branch_protections::Model>, AppError> {
    self.repository.protections(p).await
  }
  pub async fn variables(&self, p: i64) -> Result<Vec<variable::Model>, AppError> {
    self.repository.variables(p).await
  }
  pub async fn job(&self, p: i64, id: i64) -> Result<job::Model, AppError> {
    self.repository.job(p, id).await
  }
  pub async fn live_job(&self, id: i64, attempt: i32) -> Result<Option<job::Model>, AppError> {
    self.repository.live_job(id, attempt).await
  }
  pub async fn pipeline(&self, p: i64, id: i64) -> Result<pipeline::Model, AppError> {
    self.repository.pipeline(p, id).await
  }
  pub async fn runner(&self, p: i64, id: i64) -> Result<runner::Model, AppError> {
    self.repository.runner(p, id).await
  }
  pub async fn jobs(&self, p: i64) -> Result<Vec<job::Model>, AppError> {
    self.repository.jobs(p).await
  }
  pub async fn pipelines(&self, p: i64) -> Result<Vec<pipeline::Model>, AppError> {
    self.repository.pipelines(p).await
  }
  pub async fn pipeline_jobs(
    &self,
    p: i64,
    id: i64,
  ) -> Result<Vec<(link::Model, job::Model)>, AppError> {
    self.repository.pipeline_jobs(p, id).await
  }
  pub async fn job_link(&self, id: i64) -> Result<Option<link::Model>, AppError> {
    self.repository.job_link(id).await
  }
  pub async fn logs(&self, p: i64, id: i64) -> Result<Vec<log::Model>, AppError> {
    self.repository.logs(p, id).await
  }
  pub async fn artifacts(&self, p: i64, id: i64) -> Result<Vec<artifact::Model>, AppError> {
    self.repository.artifacts(p, id).await
  }
  pub async fn artifact(&self, p: i64, j: i64, id: i64) -> Result<artifact::Model, AppError> {
    self.repository.artifact(p, j, id).await
  }
  pub async fn add_artifact(&self, a: NewArtifact) -> Result<artifact::Model, AppError> {
    self.repository.add_artifact(a).await
  }
  pub async fn push_pipeline(
    &self,
    p: i64,
    branch: &str,
    sha: &str,
  ) -> Result<Option<pipeline::Model>, AppError> {
    self.repository.push_pipeline(p, branch, sha).await
  }
  pub async fn create_job(&self, input: NewJob) -> Result<job::Model, AppError> {
    if !["noop", "script"].contains(&input.kind.as_str()) || !(1..=10).contains(&input.max_attempts)
    {
      return Err(AppError::bad_request("invalid job kind or max_attempts"));
    }
    self.repository.create_job(input).await
  }
  pub async fn ready_jobs(&self, p: i64) -> Result<Vec<job::Model>, AppError> {
    self.repository.ready_jobs(p).await
  }
  pub async fn claim_job(&self, cmd: ClaimJob) -> Result<bool, AppError> {
    self
      .repository
      .claim_job(
        cmd.job_id,
        cmd.attempt,
        cmd.runner_id,
        cmd.lease_seconds,
        cmd.payload,
      )
      .await
  }
  pub async fn finish_job(&self, cmd: FinishJob) -> Result<(), AppError> {
    self.repository.finish_job(cmd).await
  }
  pub async fn cancel_job(&self, p: i64, id: i64) -> Result<(), AppError> {
    self.repository.cancel_job(p, id).await
  }
  pub async fn retry_job(&self, p: i64, id: i64) -> Result<(), AppError> {
    self.repository.retry_job(p, id).await
  }
  pub async fn refresh_job_pipeline(&self, id: i64) -> Result<(), AppError> {
    self.repository.refresh_job_pipeline(id).await
  }
  pub async fn create_pipeline(&self, input: NewPipeline) -> Result<i64, AppError> {
    self.repository.create_pipeline(input).await
  }
  pub async fn refresh_pipeline(&self, id: i64) -> Result<(), AppError> {
    self.repository.refresh_pipeline(id).await
  }
  pub async fn cancel_pipeline(&self, p: i64, id: i64) -> Result<(), AppError> {
    self.repository.cancel_pipeline(p, id).await
  }
  pub async fn retry_pipeline(&self, p: i64, id: i64) -> Result<(), AppError> {
    self.repository.retry_pipeline(p, id).await
  }
  pub async fn runners(&self, p: i64) -> Result<Vec<runner::Model>, AppError> {
    self.repository.runners(p).await
  }
  pub async fn register_runner(&self, r: NewRunner) -> Result<runner::Model, AppError> {
    if r.name.trim().is_empty() {
      return Err(AppError::bad_request("runner name is required"));
    }
    self.repository.register_runner(r).await
  }
  pub async fn remove_runner(&self, p: i64, id: i64) -> Result<runner::Model, AppError> {
    self.repository.remove_runner(p, id).await
  }
  pub async fn authenticate_runner(
    &self,
    digest: &str,
    raw: &str,
  ) -> Result<runner::Model, AppError> {
    self.repository.authenticate_runner(digest, raw).await
  }
  pub async fn heartbeat(&self, p: i64, id: i64) -> Result<Vec<job::Model>, AppError> {
    self.repository.heartbeat(p, id).await
  }
  pub async fn append_trace(&self, cmd: AppendTrace) -> Result<(), AppError> {
    self.repository.append_trace(cmd).await
  }
  pub async fn worker_tick(&self) -> Result<(), AppError> {
    self.repository.worker_tick().await
  }
}
