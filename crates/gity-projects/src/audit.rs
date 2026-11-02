use crate::RepositoryAccessRepository;
use gity_core::AppError;
use gity_entities::{project_audit_events as events, projects};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
/// Callers construct event payloads from public business data only.
pub struct AuditRecord<'a> {
  pub project: i64,
  pub actor: i64,
  pub event: &'a str,
  pub action: &'a str,
  pub target_type: &'a str,
  pub target_id: &'a str,
  pub summary: &'a str,
  pub payload: serde_json::Value,
}
impl RepositoryAccessRepository {
  pub(crate) async fn audit_events(
    &self,
    project: i64,
    limit: u64,
  ) -> Result<Vec<events::Model>, AppError> {
    Ok(
      events::Entity::find()
        .filter(events::Column::ProjectId.eq(project))
        .order_by_desc(events::Column::CreatedAt)
        .order_by_desc(events::Column::Id)
        .limit(limit)
        .all(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn record_event(&self, record: AuditRecord<'_>) -> Result<(), AppError> {
    let project = projects::Entity::find_by_id(record.project)
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("project not found"))?;
    events::ActiveModel {
      id: Set(gity_core::new_id()),
      project_id: Set(record.project),
      organization_id: Set(project.organization_id),
      event_name: Set(record.event.into()),
      action: Set(record.action.into()),
      actor_user_id: Set(record.actor),
      target_type: Set(record.target_type.into()),
      target_id: Set(record.target_id.into()),
      summary: Set(record.summary.into()),
      payload: Set(record.payload.to_string()),
      created_at: Set(chrono::Utc::now().naive_utc()),
    }
    .insert(&self.connection)
    .await?;
    Ok(())
  }
}
