use crate::{domain::*, repository::CollaborationRepository};
use gity_core::AppError;
use gity_entities::{project_issues, project_merge_requests, project_milestones as entity};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set,
  TransactionTrait, sea_query::Expr,
};
impl CollaborationRepository {
  pub async fn milestones(&self, project: i64) -> Result<Vec<entity::Model>, AppError> {
    Ok(
      entity::Entity::find()
        .filter(entity::Column::ProjectId.eq(project))
        .order_by_asc(entity::Column::Iid)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn milestone(&self, project: i64, iid: i64) -> Result<entity::Model, AppError> {
    entity::Entity::find()
      .filter(entity::Column::ProjectId.eq(project))
      .filter(entity::Column::Iid.eq(iid))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("milestone not found"))
  }
  pub async fn milestone_id(&self, project: i64, id: i64) -> Result<i64, AppError> {
    entity::Entity::find_by_id(id)
      .filter(entity::Column::ProjectId.eq(project))
      .one(&self.connection)
      .await?
      .map(|m| m.id)
      .ok_or_else(|| AppError::not_found("milestone not found"))
  }
  pub async fn create_milestone(
    &self,
    project: i64,
    command: MilestoneCreate,
  ) -> Result<entity::Model, AppError> {
    let iid = gity_persistence::next_iid(
      &self.connection,
      project,
      gity_persistence::CounterKind::Milestone,
    )
    .await?;
    let now = chrono::Utc::now().naive_utc();
    Ok(
      entity::ActiveModel {
        id: Set(gity_core::new_id()),
        project_id: Set(project),
        iid: Set(iid),
        title: Set(command.title),
        description: Set(command.description),
        state: Set(command.state),
        start_date: Set(command.start_date),
        due_date: Set(command.due_date),
        created_at: Set(now),
        updated_at: Set(now),
      }
      .insert(&self.connection)
      .await?,
    )
  }
  pub async fn update_milestone(
    &self,
    project: i64,
    iid: i64,
    changes: MilestoneChanges,
  ) -> Result<entity::Model, AppError> {
    let mut row = self.milestone(project, iid).await?.into_active_model();
    if let Change::Set(v) = changes.title {
      row.title = Set(v);
    }
    if let Change::Set(v) = changes.description {
      row.description = Set(v);
    }
    if let Change::Set(v) = changes.state {
      row.state = Set(v);
    }
    if let Change::Set(v) = changes.start_date {
      row.start_date = Set(v);
    }
    if let Change::Set(v) = changes.due_date {
      row.due_date = Set(v);
    }
    row.updated_at = Set(chrono::Utc::now().naive_utc());
    Ok(row.update(&self.connection).await?)
  }
  pub async fn delete_milestone(&self, project: i64, iid: i64) -> Result<(), AppError> {
    let row = self.milestone(project, iid).await?;
    let tx = self.connection.begin().await?;
    project_issues::Entity::update_many()
      .col_expr(
        project_issues::Column::MilestoneId,
        Expr::value(Option::<i64>::None),
      )
      .filter(project_issues::Column::ProjectId.eq(project))
      .filter(project_issues::Column::MilestoneId.eq(row.id))
      .exec(&tx)
      .await?;
    project_merge_requests::Entity::update_many()
      .col_expr(
        project_merge_requests::Column::MilestoneId,
        Expr::value(Option::<i64>::None),
      )
      .filter(project_merge_requests::Column::ProjectId.eq(project))
      .filter(project_merge_requests::Column::MilestoneId.eq(row.id))
      .exec(&tx)
      .await?;
    entity::Entity::delete_by_id(row.id).exec(&tx).await?;
    tx.commit().await?;
    Ok(())
  }
}
