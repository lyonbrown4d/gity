use crate::configuration::cfg::Config;
use crate::service::AppServices;
use fred::clients::Client;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
  pub config: Config,
  pub db_conn: DatabaseConnection,
  pub redis_client: Option<Client>,
  pub services: AppServices,
}

impl AppState {
  pub fn new(config: Config, db_conn: DatabaseConnection, redis_client: Option<Client>) -> Self {
    let services = AppServices::new(&config, db_conn.clone());
    Self {
      config,
      db_conn,
      redis_client,
      services,
    }
  }
}
