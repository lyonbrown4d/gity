use crate::repository::*;
use crate::{FinishJob, NewJob};
use gity_core::AppError;
use gity_entities::{
  project_job_logs as log, project_jobs as job, project_pipeline_jobs as link,
  project_pipelines as pipeline,
};
use sea_orm::ExprTrait;
use sea_orm::{QueryOrder, QuerySelect, Set, TransactionTrait, entity::prelude::*};
use serde_json::Value;
impl CiRepository {
  pub async fn create_job(&self, input: NewJob) -> Result<job::Model, AppError> {
    Ok(new_job(input).insert(&self.connection).await?)
  }
  pub async fn ready_jobs(&self, p: i64) -> Result<Vec<job::Model>, AppError> {
    Ok(
      job::Entity::find()
        .filter(job::Column::ProjectId.eq(p))
        .filter(job::Column::Kind.eq("script"))
        .filter(job::Column::Status.eq("pending"))
        .filter(job::Column::RunAfter.lte(time()))
        .filter(
          sea_orm::sea_query::Expr::col(job::Column::Attempts)
            .lt(sea_orm::sea_query::Expr::col(job::Column::MaxAttempts)),
        )
        .order_by_asc(job::Column::RunAfter)
        .order_by_asc(job::Column::Id)
        .limit(100)
        .all(&self.connection)
        .await?,
    )
  }
  /// Compare-and-swap the attempt and atomically persist its prepared environment.
  pub async fn claim_job(
    &self,
    id: i64,
    attempt: i32,
    runner: i64,
    lease_seconds: i64,
    payload: String,
  ) -> Result<bool, AppError> {
    let n = job::Entity::update_many()
      .set(job::ActiveModel {
        status: Set("running".into()),
        locked_by: Set(Some(format!("runner:{runner}"))),
        locked_until: Set(time() + chrono::Duration::seconds(lease_seconds.clamp(30, 3600))),
        started_at: Set(time()),
        updated_at: Set(time()),
        payload: Set(Some(payload)),
        attempts: Set(attempt + 1),
        ..Default::default()
      })
      .filter(job::Column::Id.eq(id))
      .filter(job::Column::Status.eq("pending"))
      .filter(job::Column::Attempts.eq(attempt))
      .filter(job::Column::RunAfter.lte(time()))
      .filter(
        sea_orm::sea_query::Expr::col(job::Column::Attempts)
          .lt(sea_orm::sea_query::Expr::col(job::Column::MaxAttempts)),
      )
      .exec(&self.connection)
      .await?
      .rows_affected;
    Ok(n == 1)
  }
  pub async fn finish_job(&self, cmd: FinishJob) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    let changed = job::Entity::update_many()
      .set(job::ActiveModel {
        status: Set(if cmd.success { "succeeded" } else { "failed" }.into()),
        result: Set(Some(cmd.result)),
        last_error: Set(Some(cmd.error)),
        finished_at: Set(time()),
        updated_at: Set(time()),
        locked_by: Set(Some(String::new())),
        locked_until: Set(time()),
        ..Default::default()
      })
      .filter(live(&cmd.lease))
      .exec(&tx)
      .await?
      .rows_affected;
    if changed != 1 {
      return Err(AppError::conflict("job lease changed"));
    }
    log::Entity::update_many()
      .set(log::ActiveModel {
        exit_code: Set(cmd.exit_code),
        updated_at: Set(time()),
        ..Default::default()
      })
      .filter(log::Column::ProjectJobId.eq(cmd.lease.job_id))
      .filter(log::Column::Attempt.eq(cmd.lease.attempt))
      .exec(&tx)
      .await?;
    tx.commit().await?;
    self.refresh_job_pipeline(cmd.lease.job_id).await
  }
  pub async fn cancel_job(&self, p: i64, id: i64) -> Result<(), AppError> {
    self.job(p, id).await?;
    job::Entity::update_many()
      .set(canceled())
      .filter(job::Column::ProjectId.eq(p))
      .filter(job::Column::Id.eq(id))
      .filter(job::Column::Status.is_in(["pending", "running"]))
      .exec(&self.connection)
      .await?;
    self.refresh_job_pipeline(id).await
  }
  pub async fn retry_job(&self, p: i64, id: i64) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    let pipeline_link = link::Entity::find()
      .filter(link::Column::ProjectJobId.eq(id))
      .one(&tx)
      .await?;
    // All multi-row lifecycle operations lock the pipeline before its jobs.
    if let Some(ref l) = pipeline_link {
      pipeline::Entity::update_many()
        .set(pipeline::ActiveModel {
          updated_at: Set(time()),
          ..Default::default()
        })
        .filter(pipeline::Column::Id.eq(l.pipeline_id))
        .exec(&tx)
        .await?;
    }
    let changed = job::Entity::update_many()
      .set(job::ActiveModel {
        updated_at: Set(time()),
        ..Default::default()
      })
      .filter(job::Column::Id.eq(id))
      .filter(job::Column::ProjectId.eq(p))
      .filter(job::Column::Status.is_in(["failed", "canceled"]))
      .exec(&tx)
      .await?
      .rows_affected;
    if changed != 1 {
      if job::Entity::find_by_id(id)
        .filter(job::Column::ProjectId.eq(p))
        .one(&tx)
        .await?
        .is_none()
      {
        return Err(missing());
      }
      return Err(AppError::conflict(
        "only failed or canceled jobs can be retried",
      ));
    }
    let row = job::Entity::find_by_id(id)
      .one(&tx)
      .await?
      .ok_or_else(missing)?;
    let data: Value = if row.kind == "noop" {
      serde_json::json!({})
    } else {
      serde_json::from_str(row.payload.as_deref().unwrap_or("{}"))
        .map_err(|_| AppError::internal("invalid persisted CI job payload"))?
    };
    if let Some(l) = pipeline_link {
      pipeline::Entity::update_many()
        .set(pipeline::ActiveModel {
          status: Set("pending".into()),
          finished_at: Set(zero()),
          updated_at: Set(time()),
          ..Default::default()
        })
        .filter(pipeline::Column::Id.eq(l.pipeline_id))
        .filter(pipeline::Column::Status.is_in(["failed", "canceled", "succeeded"]))
        .exec(&tx)
        .await?;
      let links = link::Entity::find()
        .filter(link::Column::PipelineId.eq(l.pipeline_id))
        .all(&tx)
        .await?;
      job::Entity::update_many()
        .set(reset(blocked(), false))
        .col_expr(
          job::Column::MaxAttempts,
          sea_orm::sea_query::Expr::col(job::Column::Attempts).add(3),
        )
        .filter(job::Column::Id.is_in(links.iter().map(|l| l.project_job_id)))
        .filter(job::Column::Id.ne(id))
        .filter(job::Column::Status.eq("canceled"))
        .exec(&tx)
        .await?;
    }
    let needs = data["needs"].as_array().is_some_and(|v| !v.is_empty());
    job::Entity::update_many()
      .set(reset(if needs { blocked() } else { time() }, true))
      .col_expr(
        job::Column::MaxAttempts,
        sea_orm::sea_query::Expr::col(job::Column::Attempts).add(3),
      )
      .filter(job::Column::Id.eq(id))
      .exec(&tx)
      .await?;
    tx.commit().await?;
    self.refresh_job_pipeline(id).await
  }
  pub async fn refresh_job_pipeline(&self, id: i64) -> Result<(), AppError> {
    if let Some(link) = self.job_link(id).await? {
      self.refresh_pipeline(link.pipeline_id).await?;
    }
    Ok(())
  }
}
pub(crate) fn canceled() -> job::ActiveModel {
  job::ActiveModel {
    status: Set("canceled".into()),
    finished_at: Set(time()),
    updated_at: Set(time()),
    locked_by: Set(Some(String::new())),
    locked_until: Set(time()),
    ..Default::default()
  }
}
pub(crate) fn reset(run_after: chrono::NaiveDateTime, clear: bool) -> job::ActiveModel {
  let mut row = job::ActiveModel {
    status: Set("pending".into()),
    locked_by: Set(Some(String::new())),
    locked_until: Set(zero()),
    run_after: Set(run_after),
    finished_at: Set(zero()),
    updated_at: Set(time()),
    ..Default::default()
  };
  if clear {
    row.result = Set(Some(String::new()));
    row.last_error = Set(Some(String::new()));
    row.started_at = Set(zero());
  }
  row
}
