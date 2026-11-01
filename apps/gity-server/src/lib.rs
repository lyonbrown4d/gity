pub mod access;
pub mod auth;
pub mod ci;
mod composition;
pub mod events;
pub mod features;
pub mod git_http;
pub mod maintenance;
pub mod observability;
pub mod openapi;
pub mod projects;
pub mod registry;
pub mod repository;
pub mod ssh;
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
    pub search: gity_search::SearchIndex,
    pub settings: Arc<Settings>,
    pub storage: gity_core::Storage,
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .merge(auth::routes())
        .merge(users::routes())
        .merge(projects::routes())
        .merge(repository::routes())
        .merge(features::routes())
        .merge(registry::routes())
        .merge(ci::routes());
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
