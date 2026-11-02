use crate::repository::CollaborationRepository;
use gity_core::AppError;
use gity_entities::{project_todos as todos, projects};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
  sea_query::OnConflict,
};

use crate::domain::Notification;
impl CollaborationRepository {
  pub async fn todos(
    &self,
    user: i64,
    state: &str,
    limit: u64,
  ) -> Result<Vec<todos::Model>, AppError> {
    let mut query = todos::Entity::find().filter(todos::Column::UserId.eq(user));
    if !state.is_empty() {
      query = query.filter(todos::Column::State.eq(state));
    }
    Ok(
      query
        .order_by_desc(todos::Column::CreatedAt)
        .order_by_desc(todos::Column::Id)
        .limit(limit)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn complete_todo(&self, user: i64, id: i64) -> Result<(), AppError> {
    let row = todos::Entity::find_by_id(id)
      .filter(todos::Column::UserId.eq(user))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("todo not found"))?;
    let now = chrono::Utc::now().naive_utc();
    todos::ActiveModel {
      id: Set(row.id),
      state: Set("done".to_owned()),
      done_at: Set(Some(now)),
      updated_at: Set(now),
      ..Default::default()
    }
    .update(&self.connection)
    .await?;
    Ok(())
  }
  pub async fn notify(&self, command: Notification) -> Result<(), AppError> {
    let project = projects::Entity::find_by_id(command.project)
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("project not found"))?;
    let now = chrono::Utc::now().naive_utc();
    let model = todos::ActiveModel {
      id: Set(gity_core::new_id()),
      user_id: Set(command.user),
      organization_id: Set(project.organization_id),
      project_id: Set(command.project),
      kind: Set(command.kind),
      state: Set("pending".to_owned()),
      target_type: Set(command.target_type),
      target_id: Set(command.target),
      title: Set(command.title),
      summary: Set(command.summary),
      action_url: Set(format!(
        "/app/projects/{}/{}{}",
        project.organization_id, project.id, command.suffix
      )),
      created_at: Set(now),
      updated_at: Set(now),
      done_at: Set(None),
    };
    todos::Entity::insert(model)
      .on_conflict(
        OnConflict::columns([
          todos::Column::UserId,
          todos::Column::Kind,
          todos::Column::TargetType,
          todos::Column::TargetId,
        ])
        .do_nothing_on([todos::Column::Id])
        .to_owned(),
      )
      .try_insert()
      .exec(&self.connection)
      .await?;
    Ok(())
  }
}
