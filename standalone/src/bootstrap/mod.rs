mod logging;

use crate::bootstrap::logging::init_logging;
use crate::configuration::cfg::{CacheType, Config};
use crate::configuration::load_config;
use crate::http::app_state::AppState;
use chrono::Utc;
use fred::clients::Client;
use fred::prelude::{Builder, ClientLike};
use migration::{Migrator, MigratorTrait};
use repository::AppRepository;
use sea_orm::{Database, DatabaseConnection};
use tokio::time::{Duration, interval};
use tracing::info;

pub async fn bootstrap() -> Result<AppState, String> {
  init_logging();

  let cfg = load_config();
  let db_conn = connect_database(&cfg).await?;

  Migrator::up(&db_conn, None)
    .await
    .map_err(|err| format!("failed to run migrations: {err}"))?;

  let redis_client = init_redis_if_enabled(&cfg).await?;
  info!("bootstrap completed");

  let app_state = AppState::new(cfg, db_conn, redis_client);

  spawn_invitation_expiry_job(app_state.db_conn.clone());

  Ok(app_state)
}

async fn connect_database(cfg: &Config) -> Result<DatabaseConnection, String> {
  Database::connect(&cfg.database.url)
    .await
    .map_err(|err| format!("failed to connect database: {err}"))
}

async fn init_redis_if_enabled(cfg: &Config) -> Result<Option<Client>, String> {
  let Some(cache) = &cfg.cache else {
    return Ok(None);
  };

  if cache.cache_type != CacheType::REDIS {
    return Ok(None);
  }

  let redis_config = fred::prelude::Config::from_url(&cache.url)
    .map_err(|err| format!("failed to parse redis url: {err}"))?;
  let client = Builder::from_config(redis_config)
    .build()
    .map_err(|err| format!("failed to build redis client: {err}"))?;

  client
    .init()
    .await
    .map_err(|err| format!("failed to init redis client: {err}"))?;

  info!("redis initialized");
  Ok(Some(client))
}

fn spawn_invitation_expiry_job(db_conn: DatabaseConnection) {
  tokio::spawn(async move {
    let mut ticker = interval(Duration::from_secs(300));

    loop {
      ticker.tick().await;

      let now = Utc::now();
      let update = AppRepository::expire_pending_invitations_before(&db_conn, now).await;

      if let Err(err) = update {
        info!("invitation expiry job failed: {err}");
      }
    }
  });
}
