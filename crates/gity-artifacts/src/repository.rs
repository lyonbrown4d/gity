use crate::{ArtifactCoordinate, PackageFormat};
use gity_core::{AppError, new_id};
use gity_entities::{project_package_files, project_package_versions, project_packages};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, PaginatorTrait,
  QueryFilter, QueryOrder, Set,
};

#[derive(Clone)]
pub struct ArtifactRepository {
  pub(crate) connection: DatabaseConnection,
}

impl ArtifactRepository {
  pub fn new(connection: DatabaseConnection) -> Self {
    Self { connection }
  }

  pub async fn find_file(
    &self,
    coordinate: &ArtifactCoordinate,
  ) -> Result<Option<project_package_files::Model>, AppError> {
    let package = project_packages::Entity::find()
      .filter(project_packages::Column::ProjectId.eq(coordinate.project_id))
      .filter(project_packages::Column::Type.eq(coordinate.format.as_str()))
      .filter(project_packages::Column::Name.eq(&coordinate.name))
      .one(&self.connection)
      .await?;
    let Some(package) = package else {
      return Ok(None);
    };
    let version = project_package_versions::Entity::find()
      .filter(project_package_versions::Column::ProjectPackageId.eq(package.id))
      .filter(project_package_versions::Column::Version.eq(&coordinate.version))
      .one(&self.connection)
      .await?;
    let Some(version) = version else {
      return Ok(None);
    };
    Ok(
      project_package_files::Entity::find()
        .filter(project_package_files::Column::ProjectPackageVersionId.eq(version.id))
        .filter(project_package_files::Column::FilePath.eq(&coordinate.path))
        .one(&self.connection)
        .await?,
    )
  }

  pub async fn find_file_by_id(
    &self,
    project_id: i64,
    file_id: i64,
  ) -> Result<Option<project_package_files::Model>, AppError> {
    let Some(file) = project_package_files::Entity::find_by_id(file_id)
      .one(&self.connection)
      .await?
    else {
      return Ok(None);
    };
    let Some(version) =
      project_package_versions::Entity::find_by_id(file.project_package_version_id)
        .one(&self.connection)
        .await?
    else {
      return Ok(None);
    };
    let Some(package) = project_packages::Entity::find_by_id(version.project_package_id)
      .one(&self.connection)
      .await?
    else {
      return Ok(None);
    };
    Ok((package.project_id == project_id).then_some(file))
  }

  pub(crate) async fn ensure_package(
    &self,
    coordinate: &ArtifactCoordinate,
  ) -> Result<project_packages::Model, AppError> {
    if let Some(package) = project_packages::Entity::find()
      .filter(project_packages::Column::ProjectId.eq(coordinate.project_id))
      .filter(project_packages::Column::Type.eq(coordinate.format.as_str()))
      .filter(project_packages::Column::Name.eq(&coordinate.name))
      .one(&self.connection)
      .await?
    {
      return Ok(package);
    }
    let now = chrono::Utc::now().naive_utc();
    let package = project_packages::ActiveModel {
      id: Set(new_id()),
      project_id: Set(coordinate.project_id),
      r#type: Set(coordinate.format.as_str().to_owned()),
      name: Set(coordinate.name.clone()),
      created_at: Set(now),
      updated_at: Set(now),
    };
    match package.insert(&self.connection).await {
      Ok(package) => Ok(package),
      Err(error) if error.sql_err().is_some() => project_packages::Entity::find()
        .filter(project_packages::Column::ProjectId.eq(coordinate.project_id))
        .filter(project_packages::Column::Type.eq(coordinate.format.as_str()))
        .filter(project_packages::Column::Name.eq(&coordinate.name))
        .one(&self.connection)
        .await?
        .ok_or_else(|| error.into()),
      Err(error) => Err(error.into()),
    }
  }

  pub(crate) async fn ensure_version(
    &self,
    package_id: i64,
    version: &str,
  ) -> Result<project_package_versions::Model, AppError> {
    if let Some(version_row) = project_package_versions::Entity::find()
      .filter(project_package_versions::Column::ProjectPackageId.eq(package_id))
      .filter(project_package_versions::Column::Version.eq(version))
      .one(&self.connection)
      .await?
    {
      return Ok(version_row);
    }
    let now = chrono::Utc::now().naive_utc();
    let version_row = project_package_versions::ActiveModel {
      id: Set(new_id()),
      project_package_id: Set(package_id),
      version: Set(version.to_owned()),
      status: Set("default".to_owned()),
      created_at: Set(now),
      updated_at: Set(now),
    };
    match version_row.insert(&self.connection).await {
      Ok(version_row) => Ok(version_row),
      Err(error) if error.sql_err().is_some() => project_package_versions::Entity::find()
        .filter(project_package_versions::Column::ProjectPackageId.eq(package_id))
        .filter(project_package_versions::Column::Version.eq(version))
        .one(&self.connection)
        .await?
        .ok_or_else(|| error.into()),
      Err(error) => Err(error.into()),
    }
  }

  pub async fn all_versions(
    &self,
    project_id: i64,
    format: PackageFormat,
    name: &str,
  ) -> Result<
    Vec<(
      project_package_versions::Model,
      Vec<project_package_files::Model>,
    )>,
    AppError,
  > {
    let Some(package) = project_packages::Entity::find()
      .filter(project_packages::Column::ProjectId.eq(project_id))
      .filter(project_packages::Column::Type.eq(format.as_str()))
      .filter(project_packages::Column::Name.eq(name))
      .one(&self.connection)
      .await?
    else {
      return Ok(Vec::new());
    };
    let versions = project_package_versions::Entity::find()
      .filter(project_package_versions::Column::ProjectPackageId.eq(package.id))
      .all(&self.connection)
      .await?;
    let mut result = Vec::with_capacity(versions.len());
    for version in versions {
      let files = project_package_files::Entity::find()
        .filter(project_package_files::Column::ProjectPackageVersionId.eq(version.id))
        .all(&self.connection)
        .await?;
      result.push((version, files));
    }
    Ok(result)
  }

  pub async fn list_packages(
    &self,
    project_id: i64,
    format: Option<PackageFormat>,
    name_contains: Option<&str>,
    page: u64,
    per_page: u64,
  ) -> Result<(Vec<project_packages::Model>, u64), AppError> {
    let mut query =
      project_packages::Entity::find().filter(project_packages::Column::ProjectId.eq(project_id));
    if let Some(format) = format {
      query = query.filter(project_packages::Column::Type.eq(format.as_str()));
    }
    if let Some(name) = name_contains.filter(|value| !value.is_empty()) {
      query = query.filter(project_packages::Column::Name.contains(name));
    }
    let paginator = query
      .order_by_desc(project_packages::Column::Id)
      .paginate(&self.connection, per_page.max(1));
    let total = paginator.num_items().await?;
    let rows = paginator.fetch_page(page.saturating_sub(1)).await?;
    Ok((rows, total))
  }

  pub async fn set_version_status(
    &self,
    project_id: i64,
    format: PackageFormat,
    name: &str,
    version_name: &str,
    status: &str,
  ) -> Result<(), AppError> {
    let package = project_packages::Entity::find()
      .filter(project_packages::Column::ProjectId.eq(project_id))
      .filter(project_packages::Column::Type.eq(format.as_str()))
      .filter(project_packages::Column::Name.eq(name))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("package not found"))?;
    let version = project_package_versions::Entity::find()
      .filter(project_package_versions::Column::ProjectPackageId.eq(package.id))
      .filter(project_package_versions::Column::Version.eq(version_name))
      .one(&self.connection)
      .await?
      .ok_or_else(|| AppError::not_found("package version not found"))?;
    let mut active = version.into_active_model();
    active.status = Set(status.to_owned());
    active.updated_at = Set(chrono::Utc::now().naive_utc());
    active.update(&self.connection).await?;
    Ok(())
  }
}
