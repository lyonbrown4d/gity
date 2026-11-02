//! Cross-project activity feed schemas.
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ActivityItem {
  /// Audit event identifier serialized as a string.
  pub id: String,
  /// Dotted event name, for example `project.issues.comment`.
  pub event_name: String,
  /// Recorded action code, for example `comment_issue`.
  pub action: String,
  pub actor_user_id: String,
  pub actor_name: String,
  pub target_type: String,
  pub target_id: String,
  pub summary: String,
  pub project_id: String,
  pub project_name: String,
  /// Owning organization identifier, used to build project links.
  pub organization_id: String,
  /// Project full path such as `team/demo-project`.
  pub project_path: String,
  #[schema(format=DateTime)]
  pub created_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct ActivityFeed {
  pub items: Vec<ActivityItem>,
}
