use crate::ArtifactRepository;
use fluxdi::module::Module;
use fluxdi::{Injector, Provider, Shared, static_module};
use gity_core::Storage;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct ArtifactService {
  pub repository: ArtifactRepository,
  pub storage: Storage,
}

impl ArtifactService {
  pub async fn cleanup_pending(&self, limit: u64) -> Result<usize, gity_core::AppError> {
    let pending = self.repository.pending_storage_deletions(limit).await?;
    let mut removed = 0;
    for row in pending {
      match self.storage.delete(&row.storage_key).await {
        Ok(()) => {
          self.repository.finish_storage_deletion(row.id).await?;
          removed += 1;
        }
        Err(error) if error.status.as_u16() == 404 => {
          self.repository.finish_storage_deletion(row.id).await?;
          removed += 1;
        }
        Err(error) => {
          tracing::warn!(error=%error, deletion_id=row.id, "artifact storage cleanup failed");
          self.repository.retry_storage_deletion(row).await?;
        }
      }
    }
    Ok(removed)
  }
}

fn service(
  connection: Shared<DatabaseConnection>,
  storage: Shared<Storage>,
) -> Result<Shared<ArtifactService>, fluxdi::Error> {
  Ok(Shared::new(ArtifactService {
    repository: ArtifactRepository::new((*connection).clone()),
    storage: (*storage).clone(),
  }))
}

static_module! {
    pub struct ArtifactModule {
        #[scope(root)]
        service:ArtifactService=service(connection,storage);
        external connection:DatabaseConnection;
        external storage:Storage;
    }
}

impl Module for ArtifactModule {
  fn configure(&self, injector: &Injector) -> Result<(), fluxdi::Error> {
    let service = self.service()?;
    injector.provide(Provider::<ArtifactService>::root(move |_| service.clone()));
    Ok(())
  }
}
