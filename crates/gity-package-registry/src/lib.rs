//! Package registry protocols backed by shared artifact storage and an authorization port.
mod module;
pub mod openapi;
mod registry;

use async_trait::async_trait;
use axum::http::HeaderMap;
pub use gity_artifacts::{
  ArtifactCoordinate, ArtifactModule, ArtifactRepository, ArtifactService, PackageFormat,
};
use gity_core::{AppError, Storage};
pub use module::{PackageRegistryHealth, PackageRegistryModule};
pub use registry::{documented_routes, package_schemas, routes};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct PackageState {
  pub artifacts: fluxdi::Shared<ArtifactService>,
  pub storage: Storage,
  pub authorization: Arc<dyn PackageAuthorization>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageAction {
  Read,
  Write,
  Delete,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackageActor {
  pub user_id: Option<i64>,
  pub source: String,
  pub pipeline_id: Option<i64>,
  pub job_id: Option<i64>,
  pub project_public: bool,
}

#[async_trait]
pub trait PackageAuthorization: Send + Sync {
  async fn authorize(
    &self,
    project_id: i64,
    action: PackageAction,
    headers: &HeaderMap,
  ) -> Result<PackageActor, AppError>;
}
