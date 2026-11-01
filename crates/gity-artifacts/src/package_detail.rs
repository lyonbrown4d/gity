use crate::ArtifactRepository;
use gity_core::AppError;
use gity_entities::{project_package_files, project_package_versions, project_packages};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

impl ArtifactRepository {
  pub async fn package_detail(
    &self,
    project_id: i64,
    package_id: i64,
  ) -> Result<
    Option<(
      project_packages::Model,
      Vec<(
        project_package_versions::Model,
        Vec<project_package_files::Model>,
      )>,
    )>,
    AppError,
  > {
    let Some(package) = project_packages::Entity::find_by_id(package_id)
      .filter(project_packages::Column::ProjectId.eq(project_id))
      .one(&self.connection)
      .await?
    else {
      return Ok(None);
    };
    let versions = project_package_versions::Entity::find()
      .filter(project_package_versions::Column::ProjectPackageId.eq(package_id))
      .order_by_desc(project_package_versions::Column::Id)
      .all(&self.connection)
      .await?;
    let mut details = Vec::with_capacity(versions.len());
    for version in versions {
      let files = project_package_files::Entity::find()
        .filter(project_package_files::Column::ProjectPackageVersionId.eq(version.id))
        .order_by_desc(project_package_files::Column::Id)
        .all(&self.connection)
        .await?;
      details.push((version, files));
    }
    Ok(Some((package, details)))
  }
}
