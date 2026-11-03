//! Current search permissions are derived from authoritative membership rows.
use crate::ProjectService;
use gity_core::{AppError, Principal};
use gity_entities::{organization_members, project_members, projects};
use gity_identity::MembershipRole;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use std::collections::HashMap;

#[derive(Clone)]
pub struct SearchProject {
  pub project: projects::Model,
  pub level: i64,
}
impl ProjectService {
  pub async fn search_scope(
    &self,
    actor: &Principal,
    organization: Option<i64>,
    selected: &[i64],
  ) -> Result<Vec<SearchProject>, AppError> {
    if selected.len() > 1000 || selected.iter().any(|id| *id <= 0) {
      return Err(AppError::bad_request("invalid search project scope"));
    }
    let db = &self.repository.connection;
    let project_roles = project_members::Entity::find()
      .filter(project_members::Column::UserId.eq(actor.user_id))
      .all(db)
      .await?
      .into_iter()
      .map(|v| (v.project_id, MembershipRole::parse(&v.role).level()))
      .filter(|(_, level)| *level > 0)
      .collect::<HashMap<_, _>>();
    let org_roles = organization_members::Entity::find()
      .filter(organization_members::Column::UserId.eq(actor.user_id))
      .all(db)
      .await?
      .into_iter()
      .map(|v| (v.organization_id, MembershipRole::parse(&v.role).level()))
      .filter(|(_, level)| *level > 0)
      .collect::<HashMap<_, _>>();
    let mut query = projects::Entity::find().filter(projects::Column::DeletedAt.is_null());
    if !actor.is_super_admin {
      query = query.filter(
        Condition::any()
          .add(projects::Column::Visibility.is_in(["public", "internal"]))
          .add(projects::Column::Id.is_in(project_roles.keys().copied().collect::<Vec<_>>()))
          .add(
            projects::Column::OrganizationId.is_in(org_roles.keys().copied().collect::<Vec<_>>()),
          ),
      );
    }
    if let Some(org) = organization {
      query = query.filter(projects::Column::OrganizationId.eq(org));
    }
    if !selected.is_empty() {
      query = query.filter(projects::Column::Id.is_in(selected.to_vec()));
    }
    let rows = query
      .order_by_asc(projects::Column::Id)
      .limit(2001)
      .all(db)
      .await?;
    if rows.len() > 2000 {
      return Err(AppError::bad_request(
        "narrow search to an organization or project",
      ));
    }
    Ok(
      rows
        .into_iter()
        .map(|project| {
          let level = if actor.is_super_admin {
            100
          } else {
            project_roles.get(&project.id).copied().unwrap_or(0).max(
              org_roles
                .get(&project.organization_id)
                .copied()
                .unwrap_or(0),
            )
          };
          SearchProject { project, level }
        })
        .collect(),
    )
  }
}
