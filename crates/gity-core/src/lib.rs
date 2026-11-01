mod config;
mod database;
mod error;
mod ids;
pub mod locale;
mod storage;
pub use config::{ObservabilityConfig, SearchConfig, Settings, SshConfig, StorageConfig};
pub use database::Db;
pub use database::audit_context;
pub use database::sql_value;
pub use error::{AppError, externalize, ok};
pub use ids::{new_id, now};
pub use storage::{Storage, StorageMeta};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Principal {
    pub user_id: i64,
    pub username: String,
    pub is_super_admin: bool,
}
