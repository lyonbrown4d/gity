use crate::{ArtifactCoordinate, ArtifactRepository};
use gity_core::{AppError, new_id};
use gity_entities::project_package_files;
use sea_orm::{ActiveModelTrait, Set};

pub struct RecordArtifactUpload<'a> {
  pub coordinate: &'a ArtifactCoordinate,
  pub file_name: &'a str,
  pub content_type: &'a str,
  pub byte_size: u64,
  pub sha256: &'a str,
  pub storage_key: &'a str,
  pub created_by_user_id: Option<i64>,
  pub source: &'a str,
  pub pipeline_id: Option<i64>,
  pub job_id: Option<i64>,
  pub registry_metadata: Option<&'a str>,
}

impl ArtifactRepository {
  pub async fn record_upload(
    &self,
    upload: RecordArtifactUpload<'_>,
  ) -> Result<(project_package_files::Model, bool), AppError> {
    let package = self.ensure_package(upload.coordinate).await?;
    let version = self
      .ensure_version(package.id, &upload.coordinate.version)
      .await?;
    let created_at = chrono::Utc::now().naive_utc();
    let file = project_package_files::ActiveModel {
      id: Set(new_id()),
      project_package_version_id: Set(version.id),
      file_name: Set(upload.file_name.to_owned()),
      file_path: Set(upload.coordinate.path.clone()),
      content_type: Set(Some(upload.content_type.to_owned())),
      byte_size: Set(
        i64::try_from(upload.byte_size)
          .map_err(|_| AppError::bad_request("package file is too large"))?,
      ),
      sha256: Set(upload.sha256.to_owned()),
      created_by_user_id: Set(upload.created_by_user_id),
      pipeline_id: Set(upload.pipeline_id),
      job_id: Set(upload.job_id),
      registry_metadata: Set(upload.registry_metadata.map(str::to_owned)),
      source: Set(upload.source.to_owned()),
      storage_key: Set(upload.storage_key.to_owned()),
      created_at: Set(created_at),
      updated_at: Set(created_at),
    };
    match file.insert(&self.connection).await {
      Ok(file) => Ok((file, true)),
      Err(error) if error.sql_err().is_some() => {
        if let Some(existing) = self.find_file(upload.coordinate).await? {
          if existing.sha256 == upload.sha256 {
            Ok((existing, false))
          } else {
            Err(AppError::conflict(
              "package file already exists with different content",
            ))
          }
        } else {
          Err(error.into())
        }
      }
      Err(error) => Err(error.into()),
    }
  }
}
