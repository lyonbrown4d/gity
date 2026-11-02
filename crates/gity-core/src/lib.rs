mod config;
mod database;
mod error;
mod ids;
pub mod locale;
mod storage;
pub use config::{
  AuthorityConfig, ComponentConfig, GitClusterConfig, GitStorageNodeConfig, MirrorUpstreamConfig,
  ObservabilityConfig, RegistryConfig, SearchConfig, Settings, SshConfig, StorageConfig,
};
pub use database::Db;
#[cfg(any(test, feature = "test-fixtures"))]
pub use database::sql_value;
pub use database::{audit_actor, audit_context};
pub use error::{AppError, externalize, normalize_timestamps, ok};
pub use ids::{IdError, IdGenerator, install, new_id, now, try_new_id};
pub use storage::{Storage, StorageMeta};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Principal {
  pub user_id: i64,
  pub username: String,
  pub is_super_admin: bool,
}
