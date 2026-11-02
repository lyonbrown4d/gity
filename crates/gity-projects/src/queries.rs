use crate::ProjectRepository;
use gity_core::AppError;
use gity_entities::{organization_members, project_members, projects, users};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
impl ProjectRepository {
  pub(crate) async fn paths(
    &self,
    candidates: Vec<String>,
  ) -> Result<Vec<projects::Model>, AppError> {
    Ok(
      projects::Entity::find()
        .filter(projects::Column::FullPath.is_in(candidates))
        .filter(projects::Column::DeletedAt.is_null())
        .all(&self.connection)
        .await?,
    )
  }

  pub(crate) async fn active_by_path(
    &self,
    path: &str,
  ) -> Result<Option<projects::Model>, AppError> {
    Ok(
      projects::Entity::find()
        .filter(projects::Column::FullPath.eq(path))
        .filter(projects::Column::DeletedAt.is_null())
        .one(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn list_after(
    &self,
    cursor: i64,
    limit: u64,
  ) -> Result<Vec<projects::Model>, AppError> {
    Ok(
      projects::Entity::find()
        .filter(projects::Column::Id.gt(cursor))
        .order_by_asc(projects::Column::Id)
        .limit(limit)
        .all(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn member_user_ids(
    &self,
    project: i64,
    organization: i64,
    organization_only: bool,
  ) -> Result<Vec<i64>, AppError> {
    let mut ids = organization_members::Entity::find()
      .filter(organization_members::Column::OrganizationId.eq(organization))
      .all(&self.connection)
      .await?
      .into_iter()
      .map(|row| row.user_id)
      .collect::<Vec<_>>();
    if !organization_only {
      ids.extend(
        project_members::Entity::find()
          .filter(project_members::Column::ProjectId.eq(project))
          .all(&self.connection)
          .await?
          .into_iter()
          .map(|row| row.user_id),
      );
    }
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
  }
  pub(crate) async fn members(
    &self,
    project: i64,
  ) -> Result<Vec<project_members::Model>, AppError> {
    Ok(
      project_members::Entity::find()
        .filter(project_members::Column::ProjectId.eq(project))
        .order_by_asc(project_members::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn organization_members(
    &self,
    org: i64,
  ) -> Result<Vec<organization_members::Model>, AppError> {
    Ok(
      organization_members::Entity::find()
        .filter(organization_members::Column::OrganizationId.eq(org))
        .order_by_asc(organization_members::Column::Id)
        .all(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn users(&self, ids: Vec<i64>) -> Result<Vec<users::Model>, AppError> {
    Ok(
      users::Entity::find()
        .filter(users::Column::Id.is_in(ids))
        .all(&self.connection)
        .await?,
    )
  }
}
