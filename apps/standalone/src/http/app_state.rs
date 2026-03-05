use crate::configuration::cfg::Config;
use crate::jobs::repository_language_job::RepositoryLanguageJobClient;
use crate::service::AppServices;
use fred::clients::Client;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
  pub config: Config,
  pub db_conn: DatabaseConnection,
  pub redis_client: Option<Client>,
  pub repository_language_jobs: Option<RepositoryLanguageJobClient>,
  pub services: AppServices,
}

impl AppState {
  pub fn new(
    config: Config,
    db_conn: DatabaseConnection,
    redis_client: Option<Client>,
    repository_language_jobs: Option<RepositoryLanguageJobClient>,
  ) -> Self {
    let services = AppServices::new(&config, db_conn.clone());
    Self {
      config,
      db_conn,
      redis_client,
      repository_language_jobs,
      services,
    }
  }
}
