//! Local identity adapter shared by the API and peer-authenticated package RPC.
use crate::{
  AppState,
  access::{self, ProjectAction},
};
use axum::http::HeaderMap;
use gity_core::AppError;
use gity_package_registry::{PackageAction, PackageActor, PackageAuthorization, PackageState};
use std::sync::Arc;

pub struct PlatformPackageAuthority {
  state: AppState,
}
impl PlatformPackageAuthority {
  pub fn new(state: AppState) -> Self {
    Self { state }
  }
}
pub fn state(state: &AppState) -> PackageState {
  PackageState {
    artifacts: state.artifacts.clone(),
    storage: state.storage.clone(),
    authorization: Arc::new(PlatformPackageAuthority::new(state.clone())),
  }
}
#[async_trait::async_trait]
impl PackageAuthorization for PlatformPackageAuthority {
  async fn authorize(
    &self,
    project_id: i64,
    action: PackageAction,
    headers: &HeaderMap,
  ) -> Result<PackageActor, AppError> {
    let write = matches!(action, PackageAction::Write);
    let job = crate::ci::package_identity(&self.state, headers, project_id, write).await?;
    if job.is_some() && matches!(action, PackageAction::Delete) {
      return Err(AppError::forbidden("CI job tokens cannot delete packages"));
    }
    let user_id = if let Some(job) = &job {
      job.actor_id
    } else {
      let required = match action {
        PackageAction::Read => ProjectAction::PackageRead,
        PackageAction::Write => ProjectAction::PackageWrite,
        PackageAction::Delete => ProjectAction::Delete,
      };
      access::project_action(&self.state, headers, project_id, required)
        .await?
        .actor()
        .ok()
        .map(|actor| actor.user_id)
    };
    let project = self
      .state
      .identity
      .active_project(project_id)
      .await?
      .ok_or_else(|| AppError::not_found("project not found"))?;
    Ok(PackageActor {
      user_id,
      source: if job.is_some() { "ci_job" } else { "manual" }.into(),
      pipeline_id: job.as_ref().and_then(|job| job.pipeline_id),
      job_id: job.map(|job| job.job_id),
      project_public: project.visibility == "public",
    })
  }
}
