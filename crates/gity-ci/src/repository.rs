use crate::NewArtifact;
use gity_core::{AppError, new_id};
use gity_entities::{
  project_branch_protections, project_ci_variables as variable, project_job_artifacts as artifact,
  project_job_logs as log, project_jobs as job, project_pipeline_jobs as link,
  project_pipelines as pipeline, project_runners as runner, projects,
};
use sea_orm::Condition;
use sea_orm::{QueryOrder, QuerySelect, Set, entity::prelude::*};

pub(crate) fn time() -> chrono::NaiveDateTime {
  chrono::Utc::now().naive_utc()
}
pub(crate) fn zero() -> chrono::NaiveDateTime {
  parse(crate::ZERO)
}
pub(crate) fn blocked() -> chrono::NaiveDateTime {
  parse(crate::BLOCKED)
}
pub(crate) fn parse(s: &str) -> chrono::NaiveDateTime {
  chrono::DateTime::parse_from_rfc3339(s)
    .expect("fixed CI timestamp")
    .naive_utc()
}
pub(crate) fn missing() -> AppError {
  AppError::not_found("CI resource not found")
}
pub(crate) fn live(lease: &crate::Lease) -> Condition {
  Condition::all()
    .add(job::Column::Id.eq(lease.job_id))
    .add(job::Column::Status.eq("running"))
    .add(job::Column::Attempts.eq(lease.attempt))
    .add(job::Column::LockedBy.eq(lease.owner.clone()))
    .add(job::Column::LockedUntil.gt(time()))
}
impl CiRepository {
  pub async fn project(&self, id: i64) -> Result<projects::Model, AppError> {
    projects::Entity::find_by_id(id)
      .filter(projects::Column::DeletedAt.is_null())
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("project not found"))
  }
  pub async fn protections(
    &self,
    p: i64,
  ) -> Result<Vec<project_branch_protections::Model>, AppError> {
    Ok(
      project_branch_protections::Entity::find()
        .filter(project_branch_protections::Column::ProjectId.eq(p))
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn variables(&self, p: i64) -> Result<Vec<variable::Model>, AppError> {
    Ok(
      variable::Entity::find()
        .filter(variable::Column::ProjectId.eq(p))
        .order_by_asc(variable::Column::Key)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn job(&self, p: i64, id: i64) -> Result<job::Model, AppError> {
    job::Entity::find_by_id(id)
      .filter(job::Column::ProjectId.eq(p))
      .one(&self.connection)
      .await?
      .ok_or_else(missing)
  }
  pub async fn live_job(&self, id: i64, attempt: i32) -> Result<Option<job::Model>, AppError> {
    Ok(
      job::Entity::find_by_id(id)
        .filter(job::Column::Status.eq("running"))
        .filter(job::Column::Attempts.eq(attempt))
        .filter(job::Column::LockedUntil.gt(time()))
        .one(&self.connection)
        .await?,
    )
  }
  pub async fn pipeline(&self, p: i64, id: i64) -> Result<pipeline::Model, AppError> {
    pipeline::Entity::find_by_id(id)
      .filter(pipeline::Column::ProjectId.eq(p))
      .one(&self.connection)
      .await?
      .ok_or_else(missing)
  }
  pub async fn runner(&self, p: i64, id: i64) -> Result<runner::Model, AppError> {
    runner::Entity::find_by_id(id)
      .filter(runner::Column::ProjectId.eq(p))
      .one(&self.connection)
      .await?
      .ok_or_else(missing)
  }
  pub async fn jobs(&self, p: i64) -> Result<Vec<job::Model>, AppError> {
    Ok(
      job::Entity::find()
        .filter(job::Column::ProjectId.eq(p))
        .order_by_desc(job::Column::CreatedAt)
        .limit(200)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn pipelines(&self, p: i64) -> Result<Vec<pipeline::Model>, AppError> {
    Ok(
      pipeline::Entity::find()
        .filter(pipeline::Column::ProjectId.eq(p))
        .order_by_desc(pipeline::Column::CreatedAt)
        .limit(200)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn pipeline_jobs(
    &self,
    p: i64,
    id: i64,
  ) -> Result<Vec<(link::Model, job::Model)>, AppError> {
    let links = link::Entity::find()
      .filter(link::Column::ProjectId.eq(p))
      .filter(link::Column::PipelineId.eq(id))
      .order_by_asc(link::Column::SortOrder)
      .all(&self.connection)
      .await?;
    let jobs = job::Entity::find()
      .filter(job::Column::Id.is_in(links.iter().map(|l| l.project_job_id)))
      .all(&self.connection)
      .await?
      .into_iter()
      .map(|j| (j.id, j))
      .collect::<std::collections::BTreeMap<_, _>>();
    Ok(
      links
        .into_iter()
        .filter_map(|l| jobs.get(&l.project_job_id).cloned().map(|j| (l, j)))
        .collect(),
    )
  }
  pub async fn job_link(&self, id: i64) -> Result<Option<link::Model>, AppError> {
    Ok(
      link::Entity::find()
        .filter(link::Column::ProjectJobId.eq(id))
        .one(&self.connection)
        .await?,
    )
  }
  pub async fn logs(&self, p: i64, id: i64) -> Result<Vec<log::Model>, AppError> {
    Ok(
      log::Entity::find()
        .filter(log::Column::ProjectId.eq(p))
        .filter(log::Column::ProjectJobId.eq(id))
        .order_by_asc(log::Column::Attempt)
        .order_by_asc(log::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn artifacts(&self, p: i64, id: i64) -> Result<Vec<artifact::Model>, AppError> {
    Ok(
      artifact::Entity::find()
        .filter(artifact::Column::ProjectId.eq(p))
        .filter(artifact::Column::ProjectJobId.eq(id))
        .order_by_asc(artifact::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn artifact(&self, p: i64, j: i64, id: i64) -> Result<artifact::Model, AppError> {
    artifact::Entity::find_by_id(id)
      .filter(artifact::Column::ProjectId.eq(p))
      .filter(artifact::Column::ProjectJobId.eq(j))
      .one(&self.connection)
      .await?
      .ok_or_else(missing)
  }
  pub async fn add_artifact(&self, a: NewArtifact) -> Result<artifact::Model, AppError> {
    Ok(
      artifact::ActiveModel {
        id: Set(a.id),
        project_id: Set(a.project_id),
        project_job_id: Set(a.job_id),
        name: Set(a.name),
        file_name: Set(a.file_name),
        file_path: Set(a.file_path),
        content_type: Set(a.content_type),
        byte_size: Set(a.byte_size),
        storage_key: Set(a.storage_key),
        created_at: Set(time()),
        updated_at: Set(time()),
      }
      .insert(&self.connection)
      .await?,
    )
  }
  pub async fn push_pipeline(
    &self,
    p: i64,
    branch: &str,
    sha: &str,
  ) -> Result<Option<pipeline::Model>, AppError> {
    Ok(
      pipeline::Entity::find()
        .filter(pipeline::Column::ProjectId.eq(p))
        .filter(pipeline::Column::Source.eq("push"))
        .filter(pipeline::Column::RefName.eq(branch))
        .filter(pipeline::Column::CommitSha.eq(sha))
        .one(&self.connection)
        .await?,
    )
  }
}
pub(crate) fn new_job(input: crate::NewJob) -> job::ActiveModel {
  job::ActiveModel {
    id: Set(new_id()),
    project_id: Set(input.project_id),
    kind: Set(input.kind),
    status: Set("pending".into()),
    payload: Set(Some(input.payload)),
    result: Set(Some(String::new())),
    attempts: Set(0),
    max_attempts: Set(input.max_attempts),
    run_after: Set(input.run_after),
    locked_by: Set(Some(String::new())),
    locked_until: Set(zero()),
    last_error: Set(Some(String::new())),
    started_at: Set(zero()),
    finished_at: Set(zero()),
    created_at: Set(time()),
    updated_at: Set(time()),
  }
}

#[derive(Clone)]
pub(crate) struct CiRepository {
  pub(crate) connection: sea_orm::DatabaseConnection,
}
