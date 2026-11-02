//! Platform side of the code-intelligence pipeline.
//!
//! The runner scans a source snapshot and uploads a symbol archive; this module
//! only decides when an index job is needed, ingests the uploaded archive into
//! `gity_code_intel::Store`, and serves the synchronous manual rebuild.
mod ingest;
mod job;
mod refresh;
mod snapshot;
#[cfg(test)]
mod tests;

pub(crate) use ingest::ingest;
pub(crate) use job::{ARTIFACT_NAME, JOB_KIND, enqueue};
pub(crate) use refresh::routes;

use gity_core::AppError;
use serde_json::Value;

fn project_id(project: &Value) -> Result<i64, AppError> {
  project["id"]
    .as_i64()
    .filter(|id| *id > 0)
    .ok_or_else(|| AppError::internal("project row is missing its identifier"))
}

/// Archive decoding and index writing are CPU and file IO; never block a worker.
async fn blocking<T: Send + 'static>(
  action: impl FnOnce() -> anyhow::Result<T> + Send + 'static,
) -> Result<T, AppError> {
  tokio::task::spawn_blocking(action)
    .await
    .map_err(|_| AppError::internal("code index worker task failed"))?
    .map_err(|error| AppError::internal(format!("code index pipeline failed: {error}")))
}
