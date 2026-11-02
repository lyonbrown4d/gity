use crate::repository::CollaborationRepository;
use gity_core::AppError;
use gity_entities::{
  organization_members, project_branch_protections, project_members, project_pipelines, projects,
  users,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

impl CollaborationRepository {
  pub async fn validate_users(&self, ids: &[i64]) -> Result<(), AppError> {
    for id in ids {
      if users::Entity::find_by_id(*id)
        .one(&self.connection)
        .await?
        .is_none()
      {
        return Err(AppError::not_found("user not found"));
      }
    }
    Ok(())
  }
  pub async fn username_id(&self, name: &str) -> Result<Option<i64>, AppError> {
    Ok(
      users::Entity::find()
        .filter(users::Column::Username.eq(name))
        .one(&self.connection)
        .await?
        .map(|u| u.id),
    )
  }
  pub async fn recipients(&self, project: i64) -> Result<Vec<i64>, AppError> {
    let p = projects::Entity::find_by_id(project)
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("project not found"))?;
    let mut ids = project_members::Entity::find()
      .filter(project_members::Column::ProjectId.eq(project))
      .all(&self.connection)
      .await?
      .into_iter()
      .map(|m| m.user_id)
      .collect::<Vec<_>>();
    ids.extend(
      organization_members::Entity::find()
        .filter(organization_members::Column::OrganizationId.eq(p.organization_id))
        .all(&self.connection)
        .await?
        .into_iter()
        .map(|m| m.user_id),
    );
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
  }
  pub async fn protections(
    &self,
    project: i64,
  ) -> Result<Vec<project_branch_protections::Model>, AppError> {
    Ok(
      project_branch_protections::Entity::find()
        .filter(project_branch_protections::Column::ProjectId.eq(project))
        .order_by_asc(project_branch_protections::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub async fn source_pipeline(
    &self,
    project: i64,
    branch: &str,
    sha: &str,
  ) -> Result<Option<project_pipelines::Model>, AppError> {
    Ok(
      project_pipelines::Entity::find()
        .filter(project_pipelines::Column::ProjectId.eq(project))
        .filter(project_pipelines::Column::RefName.eq(branch))
        .filter(project_pipelines::Column::CommitSha.eq(sha))
        .order_by_desc(project_pipelines::Column::Id)
        .one(&self.connection)
        .await?,
    )
  }
}
