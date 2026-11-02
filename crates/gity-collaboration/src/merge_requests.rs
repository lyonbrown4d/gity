use crate::{domain::*, repository::CollaborationRepository};
use gity_core::AppError;
use gity_entities::project_merge_requests as entity;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set,
  TransactionTrait, sea_query::Expr,
};

impl CollaborationRepository {
  pub async fn merge_request(&self, project: i64, key: i64) -> Result<entity::Model, AppError> {
    entity::Entity::find()
      .filter(entity::Column::ProjectId.eq(project))
      .filter(entity::Column::Iid.eq(key))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("resource not found"))
  }
  pub async fn merge_requests(&self, project: i64) -> Result<Vec<entity::Model>, AppError> {
    Ok(
      entity::Entity::find()
        .filter(entity::Column::ProjectId.eq(project))
        .order_by_desc(entity::Column::Iid)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn create_merge_request(
    &self,
    command: MergeRequestCreate,
  ) -> Result<entity::Model, AppError> {
    let iid = gity_persistence::next_iid(
      &self.connection,
      command.project_id,
      gity_persistence::CounterKind::MergeRequest,
    )
    .await?;
    let now = chrono::Utc::now().naive_utc();
    Ok(
      entity::ActiveModel {
        id: Set(gity_core::new_id()),
        project_id: Set(command.project_id),
        author_user_id: Set(command.author_user_id),
        title: Set(command.title),
        description: Set(command.description),
        source_branch: Set(command.source_branch),
        target_branch: Set(command.target_branch),
        milestone_id: Set(command.milestone_id),
        head_sha: Set(command.head_sha),
        squash: Set(command.squash),
        draft: Set(command.draft),
        iid: Set(iid),
        state: Set("opened".to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      }
      .insert(&self.connection)
      .await?,
    )
  }
  pub async fn update_merge_request(
    &self,
    project: i64,
    key: i64,
    changes: MergeRequestChanges,
  ) -> Result<entity::Model, AppError> {
    let tx = self.connection.begin().await?;
    entity::Entity::update_many()
      .col_expr(
        entity::Column::UpdatedAt,
        Expr::col(entity::Column::UpdatedAt),
      )
      .filter(entity::Column::ProjectId.eq(project))
      .filter(entity::Column::Iid.eq(key))
      .exec(&tx)
      .await?;
    let current = entity::Entity::find()
      .filter(entity::Column::ProjectId.eq(project))
      .filter(entity::Column::Iid.eq(key))
      .one(&tx)
      .await?
      .ok_or_else(|| AppError::not_found("merge request not found"))?;
    if current.active_merge_operation_id.is_some() {
      return Err(AppError::conflict(
        "merge operation is pending reconciliation",
      ));
    }
    if current.state == "merged" && matches!(&changes.state, Change::Set(_)) {
      return Err(AppError::conflict(
        "merged merge requests cannot be reopened",
      ));
    }
    let mut active = current.into_active_model();
    if let Change::Set(value) = changes.title {
      active.title = Set(value);
    }
    if let Change::Set(value) = changes.description {
      active.description = Set(value);
    }
    if let Change::Set(value) = changes.state {
      active.state = Set(value);
    }
    if let Change::Set(value) = changes.milestone_id {
      active.milestone_id = Set(value);
    }
    if let Change::Set(value) = changes.squash {
      active.squash = Set(value);
    }
    if let Change::Set(value) = changes.draft {
      active.draft = Set(value);
    }
    if let Change::Set(value) = changes.closed_at {
      active.closed_at = Set(value);
    }
    active.updated_at = Set(chrono::Utc::now().naive_utc());
    let updated = active.update(&tx).await?;
    tx.commit().await?;
    Ok(updated)
  }
}
