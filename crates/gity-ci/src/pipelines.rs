use crate::jobs::{canceled, reset};
use crate::repository::*;
use crate::{NewJob, NewPipeline, pipeline_status};
use gity_core::{AppError, new_id};
use gity_entities::{
  project_jobs as job, project_pipeline_jobs as link, project_pipelines as pipeline, projects,
};
use sea_orm::ExprTrait;
use sea_orm::{Set, TransactionTrait, entity::prelude::*};
use serde_json::json;
impl CiRepository {
  pub async fn create_pipeline(&self, input: NewPipeline) -> Result<i64, AppError> {
    let iid = gity_persistence::next_iid(
      &self.connection,
      input.project_id,
      gity_persistence::CounterKind::Pipeline,
    )
    .await?;
    let tx = self.connection.begin().await?;
    // The parent project serializes push admission on SQLite and row-locking databases.
    if input.source == "push" {
      projects::Entity::update_many()
        .col_expr(
          projects::Column::UpdatedAt,
          sea_orm::sea_query::Expr::col(projects::Column::UpdatedAt),
        )
        .filter(projects::Column::Id.eq(input.project_id))
        .exec(&tx)
        .await?;
      if let Some(row) = pipeline::Entity::find()
        .filter(pipeline::Column::ProjectId.eq(input.project_id))
        .filter(pipeline::Column::Source.eq("push"))
        .filter(pipeline::Column::RefName.eq(&input.ref_name))
        .filter(pipeline::Column::CommitSha.eq(&input.commit_sha))
        .one(&tx)
        .await?
      {
        tx.commit().await?;
        return Ok(row.id);
      }
    }
    let id = new_id();
    pipeline::ActiveModel {
      id: Set(id),
      project_id: Set(input.project_id),
      iid: Set(iid),
      name: Set(if input.name.is_empty() {
        format!("Pipeline #{iid}")
      } else {
        input.name
      }),
      source: Set(input.source),
      ref_name: Set(Some(input.ref_name)),
      commit_sha: Set(Some(input.commit_sha)),
      status: Set("pending".into()),
      config_source: Set(input.config_source),
      config_content: Set(input.config_content),
      started_at: Set(zero()),
      finished_at: Set(zero()),
      created_at: Set(time()),
      updated_at: Set(time()),
    }
    .insert(&tx)
    .await?;
    for (order, spec) in input.jobs.into_iter().enumerate() {
      let mut data = spec.payload;
      data["pipeline_id"] = json!(id);
      data["pipeline_iid"] = json!(iid);
      let row = new_job(NewJob {
        project_id: input.project_id,
        kind: "script".into(),
        payload: data.to_string(),
        max_attempts: 3,
        run_after: if spec.needs.is_empty() {
          time()
        } else {
          blocked()
        },
      })
      .insert(&tx)
      .await?;
      link::ActiveModel {
        id: Set(new_id()),
        project_id: Set(input.project_id),
        pipeline_id: Set(id),
        project_job_id: Set(row.id),
        name: Set(spec.name),
        stage: Set(spec.stage),
        needs: Set(Some(
          serde_json::to_string(&spec.needs).map_err(|_| AppError::internal("invalid needs"))?,
        )),
        image: Set(Some(spec.image)),
        script: Set(
          serde_json::to_string(&spec.script).map_err(|_| AppError::internal("invalid script"))?,
        ),
        artifacts: Set(Some(
          serde_json::to_string(&spec.artifacts)
            .map_err(|_| AppError::internal("invalid artifacts"))?,
        )),
        sort_order: Set(order as i32),
        created_at: Set(time()),
        updated_at: Set(time()),
      }
      .insert(&tx)
      .await?;
    }
    tx.commit().await?;
    Ok(id)
  }
  pub async fn refresh_pipeline(&self, id: i64) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    pipeline::Entity::update_many()
      .set(pipeline::ActiveModel {
        updated_at: Set(time()),
        ..Default::default()
      })
      .filter(pipeline::Column::Id.eq(id))
      .exec(&tx)
      .await?;
    let row = pipeline::Entity::find_by_id(id)
      .one(&tx)
      .await?
      .ok_or_else(missing)?;
    if ["canceled", "succeeded", "failed"].contains(&row.status.as_str()) {
      tx.commit().await?;
      return Ok(());
    }
    let links = link::Entity::find()
      .filter(link::Column::PipelineId.eq(id))
      .all(&tx)
      .await?;
    let jobs = job::Entity::find()
      .filter(job::Column::Id.is_in(links.iter().map(|l| l.project_job_id)))
      .all(&tx)
      .await?;
    let states = links
      .iter()
      .filter_map(|l| {
        jobs
          .iter()
          .find(|j| j.id == l.project_job_id)
          .map(|j| (l.name.as_str(), j.status.as_str()))
      })
      .collect::<std::collections::BTreeMap<_, _>>();
    let status = pipeline_status(jobs.iter().map(|j| j.status.as_str()));
    for j in &jobs {
      if ["failed", "canceled"].contains(&status)
        && ["pending", "running"].contains(&j.status.as_str())
      {
        job::Entity::update_many()
          .set(canceled())
          .filter(job::Column::Id.eq(j.id))
          .filter(job::Column::Status.is_in(["pending", "running"]))
          .exec(&tx)
          .await?;
      } else if status != "succeeded" && j.status == "pending" && j.run_after == blocked() {
        let l = links
          .iter()
          .find(|l| l.project_job_id == j.id)
          .ok_or_else(missing)?;
        let needs: Vec<String> = serde_json::from_str(l.needs.as_deref().unwrap_or("[]"))
          .map_err(|_| AppError::internal("invalid pipeline dependencies"))?;
        if needs
          .iter()
          .all(|n| states.get(n.as_str()) == Some(&"succeeded"))
        {
          job::Entity::update_many()
            .set(job::ActiveModel {
              run_after: Set(time()),
              updated_at: Set(time()),
              ..Default::default()
            })
            .filter(job::Column::Id.eq(j.id))
            .filter(job::Column::Status.eq("pending"))
            .filter(job::Column::RunAfter.eq(blocked()))
            .exec(&tx)
            .await?;
        }
      }
    }
    pipeline::Entity::update_many()
      .set(pipeline::ActiveModel {
        status: Set(status.into()),
        started_at: Set(if row.started_at == zero() && status != "pending" {
          time()
        } else {
          row.started_at
        }),
        finished_at: Set(if ["succeeded", "failed", "canceled"].contains(&status) {
          time()
        } else {
          zero()
        }),
        updated_at: Set(time()),
        ..Default::default()
      })
      .filter(pipeline::Column::Id.eq(id))
      .filter(pipeline::Column::Status.is_not_in(["canceled", "failed", "succeeded"]))
      .exec(&tx)
      .await?;
    tx.commit().await?;
    Ok(())
  }
  pub async fn cancel_pipeline(&self, p: i64, id: i64) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    pipeline::Entity::update_many()
      .set(pipeline::ActiveModel {
        status: Set("canceled".into()),
        finished_at: Set(time()),
        updated_at: Set(time()),
        ..Default::default()
      })
      .filter(pipeline::Column::Id.eq(id))
      .filter(pipeline::Column::ProjectId.eq(p))
      .filter(pipeline::Column::Status.is_in(["pending", "running"]))
      .exec(&tx)
      .await?;
    let links = link::Entity::find()
      .filter(link::Column::PipelineId.eq(id))
      .all(&tx)
      .await?;
    job::Entity::update_many()
      .set(canceled())
      .filter(job::Column::ProjectId.eq(p))
      .filter(job::Column::Id.is_in(links.iter().map(|l| l.project_job_id)))
      .filter(job::Column::Status.is_in(["pending", "running"]))
      .exec(&tx)
      .await?;
    tx.commit().await?;
    Ok(())
  }
  pub async fn retry_pipeline(&self, p: i64, id: i64) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    let changed = pipeline::Entity::update_many()
      .set(pipeline::ActiveModel {
        status: Set("pending".into()),
        started_at: Set(zero()),
        finished_at: Set(zero()),
        updated_at: Set(time()),
        ..Default::default()
      })
      .filter(pipeline::Column::ProjectId.eq(p))
      .filter(pipeline::Column::Id.eq(id))
      .filter(pipeline::Column::Status.is_in(["failed", "canceled", "succeeded"]))
      .exec(&tx)
      .await?
      .rows_affected;
    if changed != 1 {
      if pipeline::Entity::find_by_id(id)
        .filter(pipeline::Column::ProjectId.eq(p))
        .one(&tx)
        .await?
        .is_none()
      {
        return Err(missing());
      }
      return Err(AppError::conflict("pipeline must be terminal before retry"));
    }
    let links = link::Entity::find()
      .filter(link::Column::PipelineId.eq(id))
      .all(&tx)
      .await?;
    job::Entity::update_many()
      .set(reset(blocked(), true))
      .col_expr(
        job::Column::MaxAttempts,
        sea_orm::sea_query::Expr::col(job::Column::Attempts).add(3),
      )
      .filter(job::Column::Id.is_in(links.iter().map(|l| l.project_job_id)))
      .exec(&tx)
      .await?;
    tx.commit().await?;
    self.refresh_pipeline(id).await
  }
}
