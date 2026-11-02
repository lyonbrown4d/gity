use crate::NewRunner;
use crate::repository::*;
use gity_core::{AppError, new_id};
use gity_entities::{project_jobs as job, project_runners as runner};
use sea_orm::Condition;
use sea_orm::{QueryOrder, Set, entity::prelude::*};
impl CiRepository {
  pub async fn runners(&self, p: i64) -> Result<Vec<runner::Model>, AppError> {
    Ok(
      runner::Entity::find()
        .filter(runner::Column::ProjectId.eq(p))
        .order_by_asc(runner::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn register_runner(&self, r: NewRunner) -> Result<runner::Model, AppError> {
    Ok(
      runner::ActiveModel {
        id: Set(new_id()),
        project_id: Set(r.project_id),
        name: Set(r.name),
        description: Set(Some(r.description)),
        tags: Set(Some(r.tags)),
        token: Set(r.token_hash),
        status: Set("offline".into()),
        active: Set(1),
        last_contact_at: Set(zero()),
        created_at: Set(time()),
        updated_at: Set(time()),
      }
      .insert(&self.connection)
      .await?,
    )
  }
  pub async fn remove_runner(&self, p: i64, id: i64) -> Result<runner::Model, AppError> {
    let row = self.runner(p, id).await?;
    runner::Entity::delete_many()
      .filter(runner::Column::ProjectId.eq(p))
      .filter(runner::Column::Id.eq(id))
      .exec(&self.connection)
      .await?;
    Ok(row)
  }
  pub async fn authenticate_runner(
    &self,
    digest: &str,
    raw: &str,
  ) -> Result<runner::Model, AppError> {
    let row = runner::Entity::find()
      .filter(
        Condition::any().add(runner::Column::Token.eq(digest)).add(
          Condition::all()
            .add(runner::Column::Token.eq(raw))
            .add(runner::Column::Token.not_like("sha256:%")),
        ),
      )
      .filter(runner::Column::Active.eq(1))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::unauthorized("invalid runner token"))?;
    runner::Entity::update_many()
      .set(runner::ActiveModel {
        status: Set("online".into()),
        last_contact_at: Set(time()),
        updated_at: Set(time()),
        ..Default::default()
      })
      .filter(runner::Column::Id.eq(row.id))
      .exec(&self.connection)
      .await?;
    Ok(row)
  }
  pub async fn heartbeat(&self, p: i64, id: i64) -> Result<Vec<job::Model>, AppError> {
    let owner = format!("runner:{id}");
    let condition = Condition::all()
      .add(job::Column::ProjectId.eq(p))
      .add(job::Column::LockedBy.eq(owner))
      .add(job::Column::Status.eq("running"))
      .add(job::Column::LockedUntil.gt(time()));
    job::Entity::update_many()
      .set(job::ActiveModel {
        locked_until: Set(time() + chrono::Duration::seconds(180)),
        updated_at: Set(time()),
        ..Default::default()
      })
      .filter(condition.clone())
      .exec(&self.connection)
      .await?;
    Ok(
      job::Entity::find()
        .filter(condition)
        .all(&self.connection)
        .await?,
    )
  }
}
