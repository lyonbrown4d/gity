mod logging;

use crate::bootstrap::logging::init_logging;
use crate::configuration::cfg::{CacheType, Config, DatabaseType};
use crate::configuration::load_config;
use crate::http::app_state::AppState;
use crate::jobs::repository_language_job::init_repository_language_jobs;
use chrono::Utc;
use fred::clients::Client;
use fred::prelude::{Builder, ClientLike};
use migration::{Migrator, MigratorTrait};
use moka::sync::Cache;
use repository::OrganizationInvitationsRepository;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::time::{Duration, interval};
use tracing::info;

pub async fn bootstrap() -> Result<AppState, String> {
  init_logging();

  let cfg = load_config();
  validate_runtime_dependencies(&cfg)?;
  let db_conn = connect_database(&cfg).await?;

  Migrator::up(&db_conn, None)
    .await
    .map_err(|err| format!("failed to run migrations: {err}"))?;

  let cache_store = init_moka_cache(&cfg);
  let redis_client = init_redis_if_enabled(&cfg).await?;
  let repository_language_jobs = init_repository_language_jobs(&cfg, db_conn.clone()).await?;
  info!("bootstrap completed");

  let app_state = AppState::new(
    cfg,
    db_conn,
    cache_store,
    redis_client,
    repository_language_jobs,
  );

  spawn_invitation_expiry_job(app_state.db_conn.clone());

  Ok(app_state)
}

fn validate_runtime_dependencies(cfg: &Config) -> Result<(), String> {
  let issue_attachments = cfg
    .issue_attachments
    .as_ref()
    .ok_or_else(|| "issue_attachments.s3 is required in current stage".to_string())?;
  let provider = issue_attachments
    .provider
    .as_deref()
    .unwrap_or("s3")
    .trim()
    .to_ascii_lowercase();
  if provider != "s3" {
    return Err("issue_attachments.provider must be s3 in current stage".to_string());
  }

  let s3 = issue_attachments
    .s3
    .as_ref()
    .ok_or_else(|| "issue_attachments.s3 configuration is required".to_string())?;
  for (name, value) in [
    ("issue_attachments.s3.bucket", s3.bucket.as_deref()),
    ("issue_attachments.s3.access_key", s3.access_key.as_deref()),
    ("issue_attachments.s3.secret_key", s3.secret_key.as_deref()),
  ] {
    if value
      .map(str::trim)
      .filter(|item| !item.is_empty())
      .is_none()
    {
      return Err(format!("{name} is required"));
    }
  }
  Ok(())
}

async fn connect_database(cfg: &Config) -> Result<DatabaseConnection, String> {
  let url = cfg.database.url.trim();
  let url_lower = url.to_ascii_lowercase();
  let valid = match cfg.database.database_type {
    DatabaseType::SQLITE => url_lower.starts_with("sqlite:"),
    DatabaseType::MYSQL => url_lower.starts_with("mysql://"),
    DatabaseType::POSTGRES => {
      url_lower.starts_with("postgres://") || url_lower.starts_with("postgresql://")
    }
  };
  if !valid {
    return Err(format!(
      "database.url `{}` does not match configured database_type",
      cfg.database.url
    ));
  }

  let mut options = ConnectOptions::new(cfg.database.url.clone());
  if let Some(max_connections) = cfg.database.max_connections {
    options.max_connections(max_connections.max(1));
  }

  Database::connect(options)
    .await
    .map_err(|err| format!("failed to connect database: {err}"))
}

fn init_moka_cache(cfg: &Config) -> Cache<String, String> {
  let max_entries = cfg
    .cache
    .as_ref()
    .and_then(|cache| cache.moka_max_entries)
    .unwrap_or(10_000)
    .max(1);

  Cache::builder().max_capacity(max_entries).build()
}

async fn init_redis_if_enabled(cfg: &Config) -> Result<Option<Client>, String> {
  let Some(cache) = &cfg.cache else {
    return Ok(None);
  };

  if cache.cache_type != CacheType::REDIS {
    return Ok(None);
  }

  let cache_url = cache
    .url
    .as_deref()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .ok_or_else(|| "cache.url is required when cache.cache_type=REDIS".to_string())?;

  let redis_config = fred::prelude::Config::from_url(cache_url)
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
  let invitations_repository = OrganizationInvitationsRepository::new(db_conn);
  tokio::spawn(async move {
    let mut ticker = interval(Duration::from_secs(300));

    loop {
      ticker.tick().await;

      let now = Utc::now();
      let update = invitations_repository
        .expire_pending_invitations_before(now)
        .await;

      if let Err(err) = update {
        info!("invitation expiry job failed: {err}");
      }
    }
  });
}
