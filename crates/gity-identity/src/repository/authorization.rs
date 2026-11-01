use chrono::NaiveDateTime;
use gity_entities::{
  auth_sessions, organization_members, project_members, projects, user_access_tokens,
};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashMap;

use super::IdentityRepository;
use crate::{MembershipRole, ProjectAccess, UserIdentity};

impl IdentityRepository {
  pub async fn user_for_access_session(
    &self,
    digest: &str,
    now: NaiveDateTime,
  ) -> Result<Option<UserIdentity>, DbErr> {
    let Some(session) = auth_sessions::Entity::find_by_id(digest.to_owned())
      .filter(auth_sessions::Column::Kind.eq("access"))
      .filter(auth_sessions::Column::ExpiresAt.gt(now))
      .one(&self.connection)
      .await?
    else {
      return Ok(None);
    };
    self.user_by_id(session.user_id).await
  }

  pub async fn user_for_personal_token(&self, digest: &str) -> Result<Option<UserIdentity>, DbErr> {
    let Some(token) = user_access_tokens::Entity::find()
      .filter(user_access_tokens::Column::Token.eq(digest))
      .filter(user_access_tokens::Column::Name.is_not_in(["access", "refresh"]))
      .one(&self.connection)
      .await?
    else {
      return Ok(None);
    };
    self.user_by_id(token.user_id).await
  }

  pub async fn project_access(
    &self,
    project_id: i64,
    user_id: i64,
  ) -> Result<Option<ProjectAccess>, DbErr> {
    let Some(project) = projects::Entity::find_by_id(project_id)
      .filter(projects::Column::DeletedAt.is_null())
      .one(&self.connection)
      .await?
    else {
      return Ok(None);
    };
    let membership = project_members::Entity::find()
      .filter(project_members::Column::ProjectId.eq(project_id))
      .filter(project_members::Column::UserId.eq(user_id))
      .one(&self.connection)
      .await?
      .map(|m| MembershipRole::parse(&m.role))
      .unwrap_or(MembershipRole::None);
    Ok(Some(ProjectAccess {
      project_id,
      organization_id: project.organization_id,
      visibility: project.visibility,
      membership,
    }))
  }

  pub async fn active_project(&self, project_id: i64) -> Result<Option<projects::Model>, DbErr> {
    projects::Entity::find_by_id(project_id)
      .filter(projects::Column::DeletedAt.is_null())
      .one(&self.connection)
      .await
  }

  pub async fn active_projects(&self) -> Result<Vec<projects::Model>, DbErr> {
    projects::Entity::find()
      .filter(projects::Column::DeletedAt.is_null())
      .order_by_desc(projects::Column::Id)
      .all(&self.connection)
      .await
  }

  pub async fn organization_role(
    &self,
    organization_id: i64,
    user_id: i64,
  ) -> Result<MembershipRole, DbErr> {
    Ok(
      organization_members::Entity::find()
        .filter(organization_members::Column::OrganizationId.eq(organization_id))
        .filter(organization_members::Column::UserId.eq(user_id))
        .one(&self.connection)
        .await?
        .map(|m| MembershipRole::parse(&m.role))
        .unwrap_or(MembershipRole::None),
    )
  }

  pub async fn organization_roles(
    &self,
    user_id: i64,
  ) -> Result<HashMap<i64, MembershipRole>, DbErr> {
    Ok(
      organization_members::Entity::find()
        .filter(organization_members::Column::UserId.eq(user_id))
        .all(&self.connection)
        .await?
        .into_iter()
        .map(|member| (member.organization_id, MembershipRole::parse(&member.role)))
        .collect(),
    )
  }

  pub async fn project_role(&self, project_id: i64, user_id: i64) -> Result<MembershipRole, DbErr> {
    Ok(
      project_members::Entity::find()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::UserId.eq(user_id))
        .one(&self.connection)
        .await?
        .map(|m| MembershipRole::parse(&m.role))
        .unwrap_or(MembershipRole::None),
    )
  }
}
