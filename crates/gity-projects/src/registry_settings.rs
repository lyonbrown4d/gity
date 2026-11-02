use crate::RepositoryAccessRepository;
use gity_core::AppError;
use gity_entities::project_registry_settings as settings;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set, TransactionTrait};
impl RepositoryAccessRepository {
  pub(crate) async fn registry_settings(
    &self,
    project: i64,
  ) -> Result<Option<settings::Model>, AppError> {
    Ok(
      settings::Entity::find_by_id(project)
        .one(&self.connection)
        .await?,
    )
  }
  pub(crate) async fn set_registry_settings(
    &self,
    project: i64,
    mode: &str,
    alias: Option<String>,
    prefix: String,
    actor: i64,
  ) -> Result<(), AppError> {
    let tx = self.connection.begin().await?;
    crate::lfs::lock_project(&tx, project).await?;
    let existing = settings::Entity::find_by_id(project).one(&tx).await?;
    let update = existing.is_some();
    let mut model = existing
      .map(|row| row.into_active_model())
      .unwrap_or_else(|| settings::ActiveModel {
        project_id: Set(project),
        ..Default::default()
      });
    model.mode = Set(mode.into());
    model.mirror_alias = Set(alias);
    model.mirror_repository_prefix = Set(prefix);
    model.updated_by = Set(Some(actor));
    model.updated_at = Set(chrono::Utc::now().naive_utc());
    if update {
      model.update(&tx).await?;
    } else {
      model.insert(&tx).await?;
    }
    tx.commit().await?;
    Ok(())
  }
}
