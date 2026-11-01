use crate::ArtifactRepository;
use gity_core::{AppError, new_id};
use gity_entities::{
  artifact_storage_deletions, project_package_files, project_package_versions, project_packages,
};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set, SqlErr,
  TransactionTrait,
};

impl ArtifactRepository {
  pub async fn delete_file(&self, project_id: i64, file_id: i64) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    let file = project_package_files::Entity::find_by_id(file_id)
      .one(&tx)
      .await?
      .ok_or_else(|| AppError::not_found("package file not found"))?;
    let version = project_package_versions::Entity::find_by_id(file.project_package_version_id)
      .one(&tx)
      .await?
      .ok_or_else(|| AppError::not_found("package version not found"))?;
    let package = project_packages::Entity::find_by_id(version.project_package_id)
      .one(&tx)
      .await?
      .ok_or_else(|| AppError::not_found("package not found"))?;
    if package.project_id != project_id {
      return Err(AppError::not_found("package file not found"));
    }
    enqueue_cleanup(&tx, &file.storage_key).await?;
    project_package_files::Entity::delete_by_id(file_id)
      .exec(&tx)
      .await?;
    tx.commit().await?;
    Ok(())
  }

  pub async fn delete_version(
    &self,
    project_id: i64,
    package_id: i64,
    version_name: &str,
  ) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    let package = project_packages::Entity::find_by_id(package_id)
      .one(&tx)
      .await?
      .filter(|package| package.project_id == project_id)
      .ok_or_else(|| AppError::not_found("package not found"))?;
    let version = project_package_versions::Entity::find()
      .filter(project_package_versions::Column::ProjectPackageId.eq(package.id))
      .filter(project_package_versions::Column::Version.eq(version_name))
      .one(&tx)
      .await?
      .ok_or_else(|| AppError::not_found("package version not found"))?;
    let files = project_package_files::Entity::find()
      .filter(project_package_files::Column::ProjectPackageVersionId.eq(version.id))
      .all(&tx)
      .await?;
    for file in &files {
      enqueue_cleanup(&tx, &file.storage_key).await?;
    }
    project_package_versions::Entity::delete_by_id(version.id)
      .exec(&tx)
      .await?;
    tx.commit().await?;
    Ok(())
  }

  pub async fn delete_package(&self, project_id: i64, package_id: i64) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    let package = project_packages::Entity::find_by_id(package_id)
      .one(&tx)
      .await?
      .filter(|package| package.project_id == project_id)
      .ok_or_else(|| AppError::not_found("package not found"))?;
    let versions = project_package_versions::Entity::find()
      .filter(project_package_versions::Column::ProjectPackageId.eq(package.id))
      .all(&tx)
      .await?;
    for version in versions {
      let files = project_package_files::Entity::find()
        .filter(project_package_files::Column::ProjectPackageVersionId.eq(version.id))
        .all(&tx)
        .await?;
      for file in &files {
        enqueue_cleanup(&tx, &file.storage_key).await?;
      }
    }
    project_packages::Entity::delete_by_id(package.id)
      .exec(&tx)
      .await?;
    tx.commit().await?;
    Ok(())
  }

  pub async fn pending_storage_deletions(
    &self,
    limit: u64,
  ) -> Result<Vec<artifact_storage_deletions::Model>, AppError> {
    Ok(
      artifact_storage_deletions::Entity::find()
        .order_by_asc(artifact_storage_deletions::Column::CreatedAt)
        .limit(limit)
        .all(&self.connection)
        .await?,
    )
  }

  pub async fn finish_storage_deletion(&self, id: i64) -> Result<(), AppError> {
    artifact_storage_deletions::Entity::delete_by_id(id)
      .exec(&self.connection)
      .await?;
    Ok(())
  }

  pub async fn retry_storage_deletion(
    &self,
    row: artifact_storage_deletions::Model,
  ) -> Result<(), AppError> {
    let active = artifact_storage_deletions::ActiveModel {
      id: Set(row.id),
      storage_key: Set(row.storage_key),
      attempts: Set(row.attempts + 1),
      created_at: Set(row.created_at),
    };
    active.update(&self.connection).await?;
    Ok(())
  }
}

async fn enqueue_cleanup(
  tx: &sea_orm::DatabaseTransaction,
  storage_key: &str,
) -> Result<(), AppError> {
  let active = artifact_storage_deletions::ActiveModel {
    id: Set(new_id()),
    storage_key: Set(storage_key.to_owned()),
    attempts: Set(0),
    created_at: Set(chrono::Utc::now().naive_utc()),
  };
  match active.insert(tx).await {
    Ok(_) => Ok(()),
    Err(error) if matches!(error.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) => Ok(()),
    Err(error) => Err(error.into()),
  }
}
