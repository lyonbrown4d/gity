use super::session::Session;
use crate::AppState;
use gity_persistence::search_jobs::Jobs;
use gity_search::{BackendKind, GlobalIndex, GlobalSearchConfig};
use std::{collections::HashMap, sync::Mutex};

pub struct GlobalSearchRuntime {
  engine: Option<GlobalIndex>,
  pub(crate) jobs: Jobs,
  pub(crate) namespace_prefix: String,
  pub(crate) sessions: Mutex<HashMap<String, Session>>,
  pub(crate) query_slots: tokio::sync::Semaphore,
}
impl GlobalSearchRuntime {
  pub(crate) fn engine(&self) -> Result<&GlobalIndex, gity_core::AppError> {
    self
      .engine
      .as_ref()
      .ok_or_else(|| gity_core::AppError::conflict("global search is disabled"))
  }
  pub fn new(settings: &gity_core::Settings, db: &gity_core::Db) -> anyhow::Result<Self> {
    let cfg = &settings.search.global;
    anyhow::ensure!(cfg.validate(), "invalid global search configuration");
    let config = GlobalSearchConfig {
      backend: if cfg.backend == "opensearch" {
        BackendKind::OpenSearch
      } else {
        BackendKind::Tantivy
      },
      endpoint: cfg.endpoint.clone(),
      index: cfg.index.clone(),
      username: cfg.username.clone(),
      password: cfg.password.clone(),
      timeout_seconds: cfg.timeout_seconds,
      allow_insecure_http: cfg.allow_insecure_http,
    };
    let root = settings.search.index_root.join("global-v1");
    let engine = if cfg.enabled {
      Some(GlobalIndex::new(&root, config)?)
    } else {
      None
    };
    // Embedded roots and remote clusters must never consume each other's
    // publication tokens after a configuration change or volume replacement.
    let identity = if cfg.enabled && cfg.backend == "tantivy" {
      let path = root.join("instance-id");
      if !path.exists() {
        std::fs::write(&path, uuid::Uuid::now_v7().to_string())?;
      }
      std::fs::read_to_string(path)?
    } else {
      format!(
        "opensearch:{}:{}",
        cfg.endpoint.trim_end_matches('/'),
        cfg.index
      )
    };
    use sha2::{Digest, Sha256};
    let namespace_prefix = format!("{}-", hex::encode(Sha256::digest(identity.as_bytes())));
    Ok(Self {
      engine,
      jobs: Jobs::with_namespace(db.connection.clone(), namespace_prefix.clone()),
      namespace_prefix,
      sessions: Mutex::new(HashMap::new()),
      query_slots: tokio::sync::Semaphore::new(8),
    })
  }
}
pub(crate) fn enabled(state: &AppState) -> Result<(), gity_core::AppError> {
  if !state.settings.search.global.enabled {
    return Err(gity_core::AppError::conflict("global search is disabled"));
  }
  Ok(())
}
