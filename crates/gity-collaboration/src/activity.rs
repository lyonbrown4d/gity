//! Cross-project activity feed queries for one authenticated user.
//!
//! The feed only exposes audit events from projects the user can already read,
//! so it never becomes a side channel for private project activity.
use crate::repository::CollaborationRepository;
use gity_core::AppError;
use gity_entities::{organization_members, project_audit_events, project_members, projects, users};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use std::collections::HashMap;

/// One audit event plus the display names the feed needs.
///
/// `project_name` and `actor_name` are resolved in the same query pass so the
/// HTTP layer never performs its own lookups.
pub struct ActivityItem {
  pub event: project_audit_events::Model,
  pub project_name: String,
  pub project_path: String,
  pub actor_name: String,
}

impl CollaborationRepository {
  /// Newest audit events from every project the user may read.
  ///
  /// Visibility matches the project read rule: explicit project membership,
  /// membership of the owning organization, or a public project. Deleted
  /// projects are excluded, and results are ordered newest first and truncated
  /// to `limit` after filtering.
  pub async fn activity(&self, user: i64, limit: u64) -> Result<Vec<ActivityItem>, AppError> {
    let visible = self.visible_projects(user).await?;
    if visible.is_empty() {
      return Ok(Vec::new());
    }
    let names = visible
      .iter()
      .map(|project| {
        (
          project.id,
          (project.name.clone(), project.full_path.clone()),
        )
      })
      .collect::<HashMap<_, _>>();
    let events = project_audit_events::Entity::find()
      .filter(
        project_audit_events::Column::ProjectId.is_in(names.keys().copied().collect::<Vec<_>>()),
      )
      .order_by_desc(project_audit_events::Column::CreatedAt)
      .order_by_desc(project_audit_events::Column::Id)
      .limit(limit)
      .all(&self.connection)
      .await?;
    let accounts = self.actor_names(&events).await?;
    Ok(
      events
        .into_iter()
        .map(|event| {
          let (project_name, project_path) =
            names.get(&event.project_id).cloned().unwrap_or_default();
          ActivityItem {
            project_name,
            project_path,
            actor_name: accounts
              .get(&event.actor_user_id)
              .cloned()
              .unwrap_or_default(),
            event,
          }
        })
        .collect(),
    )
  }
  async fn visible_projects(&self, user: i64) -> Result<Vec<projects::Model>, AppError> {
    let member = project_members::Entity::find()
      .filter(project_members::Column::UserId.eq(user))
      .all(&self.connection)
      .await?
      .into_iter()
      .map(|row| row.project_id)
      .collect::<Vec<_>>();
    let organizations = organization_members::Entity::find()
      .filter(organization_members::Column::UserId.eq(user))
      .all(&self.connection)
      .await?
      .into_iter()
      .map(|row| row.organization_id)
      .collect::<Vec<_>>();
    let mut scope = Condition::any().add(projects::Column::Visibility.eq("public"));
    if !member.is_empty() {
      scope = scope.add(projects::Column::Id.is_in(member));
    }
    if !organizations.is_empty() {
      scope = scope.add(projects::Column::OrganizationId.is_in(organizations));
    }
    Ok(
      projects::Entity::find()
        .filter(projects::Column::DeletedAt.is_null())
        .filter(scope)
        .all(&self.connection)
        .await?,
    )
  }
  async fn actor_names(
    &self,
    events: &[project_audit_events::Model],
  ) -> Result<HashMap<i64, String>, AppError> {
    let mut ids = events
      .iter()
      .map(|event| event.actor_user_id)
      .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
      return Ok(HashMap::new());
    }
    Ok(
      users::Entity::find()
        .filter(users::Column::Id.is_in(ids))
        .all(&self.connection)
        .await?
        .into_iter()
        .map(|user| {
          let name = if user.display_name.trim().is_empty() {
            user.username
          } else {
            user.display_name
          };
          (user.id, name)
        })
        .collect(),
    )
  }
}
