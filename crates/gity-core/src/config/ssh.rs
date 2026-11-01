use crate::AppError;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Deserialize)]
pub struct SshConfig {
    pub enabled: bool,
    pub address: String,
    pub host_key: PathBuf,
    pub max_connections: usize,
    pub operation_timeout_seconds: u64,
}
impl SshConfig {
    pub(super) fn validate(&self) -> Result<(), AppError> {
        if self.address.parse::<std::net::SocketAddr>().is_err()
            || self.host_key.as_os_str().is_empty()
            || !(1..=1024).contains(&self.max_connections)
            || !(1..=86400).contains(&self.operation_timeout_seconds)
        {
            return Err(AppError::bad_request("invalid SSH listener configuration"));
        }
        Ok(())
    }
}
