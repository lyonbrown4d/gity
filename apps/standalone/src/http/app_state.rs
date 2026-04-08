use crate::configuration::cfg::Config;
use crate::jobs::repository_language_job::RepositoryLanguageJobClient;
use crate::service::{AppServices, ProjectSpaceService};
use fred::clients::Client;
use moka::sync::Cache;
use platform::database::DatabaseRuntime;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
  pub config: Config,
  pub db_conn: DatabaseConnection,
  pub cache_store: Cache<String, String>,
  pub redis_client: Option<Client>,
  pub repository_language_jobs: Option<RepositoryLanguageJobClient>,
  pub services: AppServices,
  pub project_space: ProjectSpaceService,
}

impl AppState {
  pub fn new(
    config: Config,
    db_conn: DatabaseConnection,
    project_space_runtime: DatabaseRuntime,
    cache_store: Cache<String, String>,
    redis_client: Option<Client>,
    repository_language_jobs: Option<RepositoryLanguageJobClient>,
  ) -> Self {
    let services = AppServices::new(&config, db_conn.clone());
    let project_space =
      ProjectSpaceService::new(project_space_runtime.db(), services.git_backend.clone());
    Self {
      config,
      db_conn,
      cache_store,
      redis_client,
      repository_language_jobs,
      services,
      project_space,
    }
  }
}
