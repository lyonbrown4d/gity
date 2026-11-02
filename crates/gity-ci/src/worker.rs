use crate::repository::*;
use gity_core::AppError;
use gity_entities::{
  project_jobs as job, project_pipelines as pipeline, project_runners as runner,
};
use sea_orm::ExprTrait;
use sea_orm::{QueryOrder, QuerySelect, Set, entity::prelude::*};
impl CiRepository {
  pub async fn worker_tick(&self) -> Result<(), AppError> {
    job::Entity::update_many()
      .set(job::ActiveModel {
        status: Set("succeeded".into()),
        result: Set(Some("noop completed".into())),
        started_at: Set(time()),
        finished_at: Set(time()),
        updated_at: Set(time()),
        ..Default::default()
      })
      .col_expr(
        job::Column::Attempts,
        sea_orm::sea_query::Expr::col(job::Column::Attempts).add(1),
      )
      .filter(job::Column::Kind.eq("noop"))
      .filter(job::Column::Status.eq("pending"))
      .filter(job::Column::RunAfter.lte(time()))
      .exec(&self.connection)
      .await?;
    // Compare the attempt and current expiry: a renewed lease is no longer expired.
    let expired = job::Entity::find()
      .filter(job::Column::Kind.is_in(["script", "index"]))
      .filter(job::Column::Status.eq("running"))
      .filter(job::Column::LockedUntil.lt(time()))
      .all(&self.connection)
      .await?;
    for row in expired {
      job::Entity::update_many()
        .set(job::ActiveModel {
          status: Set(
            if row.attempts >= row.max_attempts {
              "failed"
            } else {
              "pending"
            }
            .into(),
          ),
          last_error: Set(Some("runner lease expired".into())),
          locked_by: Set(Some(String::new())),
          locked_until: Set(time()),
          run_after: Set(time()),
          updated_at: Set(time()),
          ..Default::default()
        })
        .filter(job::Column::Id.eq(row.id))
        .filter(job::Column::Status.eq("running"))
        .filter(job::Column::Attempts.eq(row.attempts))
        .filter(job::Column::LockedUntil.lt(time()))
        .exec(&self.connection)
        .await?;
    }
    runner::Entity::update_many()
      .set(runner::ActiveModel {
        status: Set("offline".into()),
        ..Default::default()
      })
      .filter(runner::Column::LastContactAt.lt(time() - chrono::Duration::minutes(2)))
      .exec(&self.connection)
      .await?;
    for row in pipeline::Entity::find()
      .filter(pipeline::Column::Status.is_in(["pending", "running"]))
      .order_by_asc(pipeline::Column::CreatedAt)
      .limit(200)
      .all(&self.connection)
      .await?
    {
      self.refresh_pipeline(row.id).await?;
    }
    Ok(())
  }
}
