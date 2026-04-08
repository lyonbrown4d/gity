use crate::config::DatabaseConfig;
use toasty::Db;
use toasty_driver_postgresql::PostgreSQL;

#[derive(Debug, Clone)]
pub struct DatabaseRuntime {
  config: DatabaseConfig,
  db: Db,
}

impl DatabaseRuntime {
  pub async fn connect(config: DatabaseConfig) -> Result<Self, String> {
    let driver = PostgreSQL::new(config.url.clone())
      .map_err(|err| format!("failed to build toasty postgres driver: {err}"))?;
    let db = Db::builder()
      .models(toasty::models!(models::*))
      .build(driver)
      .await
      .map_err(|err| format!("failed to initialize toasty db: {err}"))?;
    db.push_schema()
      .await
      .map_err(|err| format!("failed to push toasty schema: {err}"))?;

    Ok(Self { config, db })
  }

  pub fn config(&self) -> &DatabaseConfig {
    &self.config
  }

  pub fn db(&self) -> Db {
    self.db.clone()
  }
}
