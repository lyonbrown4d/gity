use crate::configuration::cfg::Config;
use fred::clients::Client;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
  pub config: Config,
  pub db_conn: DatabaseConnection,
  pub redis_client: Option<Client>,
}

