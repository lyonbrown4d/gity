use crate::configuration;
use crate::configuration::cfg::Config;
use fred::clients::Client;
use fred::prelude::{Builder, ClientLike, EventInterface, TcpConfig};
use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;
use std::time::Duration;
use tracing::info;

#[derive(Clone, Debug)]
pub struct AppState {
  pub config: Config,
  pub db_conn: DatabaseConnection,
  pub redis_client: Client,
}

fn init_logging() {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_test_writer()
    .init();
}

fn init_env() -> Config {
  dotenvy::dotenv().ok();
  let cfg = configuration::loader::load();
  let cfg_json = serde_json::to_string_pretty(&cfg).unwrap();
  info!("Loaded configuration:\n{}", cfg_json);
  cfg
}

pub async fn init() -> AppState {
  init_logging();
  let cfg = init_env();
  let connection = sea_orm::Database::connect("postgres://root:root@localhost:5432/gity")
    .await
    .unwrap();

  let config = fred::prelude::Config::from_url("redis://localhost:6379/1").unwrap();
  let client = Builder::from_config(config)
    .with_connection_config(|config| {
      config.connection_timeout = Duration::from_secs(5);
      config.tcp = TcpConfig {
        nodelay: Some(true),
        ..Default::default()
      };
    })
    .build()
    .unwrap();
  client.init().await.unwrap();
  client.on_error(|(error, server)| async move {
    println!("{:?}: Connection error: {:?}", server, error);
    Ok(())
  });
  Migrator::down(&connection, None).await.unwrap();
  Migrator::up(&connection, None).await.unwrap();
  AppState {
    config: cfg,
    db_conn: connection,
    redis_client: client,
  }
}
