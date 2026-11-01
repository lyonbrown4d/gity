use crate::ArtifactRepository;
use gity_core::{AppError, new_id};
use gity_entities::{project_npm_dist_tags, project_packages};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};

impl ArtifactRepository {
  pub async fn npm_dist_tags(
    &self,
    project_id: i64,
    package_name: &str,
  ) -> Result<Vec<(String, String)>, AppError> {
    let Some(package) = project_packages::Entity::find()
      .filter(project_packages::Column::ProjectId.eq(project_id))
      .filter(project_packages::Column::Type.eq("npm"))
      .filter(project_packages::Column::Name.eq(package_name))
      .one(&self.connection)
      .await?
    else {
      return Ok(Vec::new());
    };
    Ok(
      project_npm_dist_tags::Entity::find()
        .filter(project_npm_dist_tags::Column::ProjectPackageId.eq(package.id))
        .all(&self.connection)
        .await?
        .into_iter()
        .map(|tag| (tag.tag, tag.version))
        .collect(),
    )
  }

  pub async fn set_npm_dist_tag(
    &self,
    project_id: i64,
    package_name: &str,
    tag: &str,
    version: Option<&str>,
  ) -> Result<(), AppError> {
    let Some(package) = project_packages::Entity::find()
      .filter(project_packages::Column::ProjectId.eq(project_id))
      .filter(project_packages::Column::Type.eq("npm"))
      .filter(project_packages::Column::Name.eq(package_name))
      .one(&self.connection)
      .await?
    else {
      return Err(AppError::not_found("npm package not found"));
    };
    let current = project_npm_dist_tags::Entity::find()
      .filter(project_npm_dist_tags::Column::ProjectPackageId.eq(package.id))
      .filter(project_npm_dist_tags::Column::Tag.eq(tag))
      .one(&self.connection)
      .await?;
    if let Some(version) = version {
      let now = chrono::Utc::now().naive_utc();
      if let Some(current) = current {
        let mut active = current.into_active_model();
        active.version = Set(version.to_owned());
        active.updated_at = Set(now);
        active.update(&self.connection).await?;
      } else {
        project_npm_dist_tags::ActiveModel {
          id: Set(new_id()),
          project_package_id: Set(package.id),
          tag: Set(tag.to_owned()),
          version: Set(version.to_owned()),
          created_at: Set(now),
          updated_at: Set(now),
        }
        .insert(&self.connection)
        .await?;
      }
    } else if let Some(current) = current {
      project_npm_dist_tags::Entity::delete_by_id(current.id)
        .exec(&self.connection)
        .await?;
    }
    Ok(())
  }
}
