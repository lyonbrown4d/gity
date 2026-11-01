pub mod access;
pub mod application;
pub mod auth;
pub mod ci;
mod composition;
pub mod deployment_api;
pub mod events;
pub mod features;
pub mod git_http;
pub mod lifecycle;
pub mod maintenance;
pub mod observability;
#[cfg(feature = "oci")]
pub mod oci_registry;
pub mod openapi;
#[cfg(feature = "packages")]
pub mod package_authority;
pub mod projects;
pub mod registry;
pub mod repository;
pub mod ssh;
pub mod standalone;
pub mod users;
pub use composition::compose;
#[cfg(test)]
mod auth_tests;

use axum::Router;
use gity_core::{Db, Settings};
use gity_git::GitService;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
pub struct AppState {
  pub db: Db,
  pub git: GitService,
  pub git_cluster: Option<gity_git_cluster::GitClusterModule>,
  pub search: gity_search::SearchIndex,
  pub settings: Arc<Settings>,
  pub storage: gity_core::Storage,
  pub identity: fluxdi::Shared<gity_identity::IdentityRepository>,
  #[cfg(feature = "packages")]
  pub artifacts: fluxdi::Shared<gity_artifacts::ArtifactService>,
}

pub fn router(state: AppState) -> Router {
  let api = Router::new()
    .merge(deployment_api::documented_routes().split_for_parts().0)
    .merge(auth::routes())
    .merge(users::routes())
    .merge(projects::routes())
    .merge(repository::routes())
    .merge(features::routes())
    .merge(registry::routes())
    .merge(ci::routes());
  #[cfg(feature = "packages")]
  let api = api.merge(
    gity_package_registry::routes().with_state::<AppState>(package_authority::state(&state)),
  );
  #[cfg(feature = "oci")]
  let api = api.merge(oci_registry::documented_routes().split_for_parts().0);
  Router::new()
    .merge(openapi::system_routes().split_for_parts().0)
    .merge(openapi::routes())
    .nest("/api/v1", api)
    .merge(git_http::routes())
    .fallback(fallback)
    .layer(axum::middleware::from_fn_with_state(
      state.clone(),
      observability::request,
    ))
    .with_state(state)
}
async fn fallback(
  axum::extract::State(state): axum::extract::State<AppState>,
  request: axum::extract::Request,
) -> Result<axum::response::Response, gity_core::AppError> {
  use axum::response::IntoResponse;
  use tower::ServiceExt;
  if registry::is_lfs_path(request.uri().path()) {
    return registry::handle_lfs(state, request).await;
  }
  if request.uri().path().starts_with("/api/") || request.uri().path().contains(".git/") {
    return Err(gity_core::AppError::not_found("endpoint not found"));
  }
  let web = ServeDir::new(&state.settings.web_root)
    .not_found_service(ServeFile::new(state.settings.web_root.join("index.html")));
  let response = web
    .oneshot(request)
    .await
    .unwrap_or_else(|error| match error {});
  Ok(response.into_response())
}
