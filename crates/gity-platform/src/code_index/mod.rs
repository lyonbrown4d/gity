//! Platform side of the code-intelligence pipeline.
//!
//! The runner scans a source snapshot and uploads a symbol archive; this module
//! decides when an index job is needed, ingests the uploaded archive into
//! `gity_code_intel::Store`, and serves the synchronous manual rebuild. On top of
//! that, [`remote`] mirrors every archive into object storage so replicas that
//! never ran the ingest can still serve the precise layer.
mod ingest;
mod job;
mod refresh;
mod remote;
mod snapshot;
#[cfg(test)]
mod tests;
#[cfg(test)]
#[path = "tests/remote.rs"]
mod tests_remote;

pub(crate) use ingest::ingest;
pub(crate) use job::{ARTIFACT_NAME, JOB_KIND, enqueue};
pub(crate) use refresh::routes;
pub(crate) use remote::{hydrate, hydrate_head, publish, publish_archive};

use crate::AppState;
use gity_core::AppError;
use serde_json::Value;

fn project_id(project: &Value) -> Result<i64, AppError> {
  project["id"]
    .as_i64()
    .filter(|id| *id > 0)
    .ok_or_else(|| AppError::internal("project row is missing its identifier"))
}

/// What the local store currently serves for a project, if anything.
pub(crate) async fn local_status(
  s: &AppState,
  project_id: i64,
) -> Result<Option<gity_code_intel::Status>, AppError> {
  let store = s.code_index.clone();
  blocking(move || store.status(project_id)).await
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
