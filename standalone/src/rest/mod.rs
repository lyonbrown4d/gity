use crate::app_state::AppState;
use crate::rest::auth::auth_routes;
use crate::rest::openapi::{openapi, ApiDoc};

use crate::rest::middleware::apply_request_id_middleware;
use crate::rest::user::user_routes;
use axum::routing::get;
use axum::Router;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
// Swagger UI removed to avoid build-time downloads

pub mod auth;
pub mod middleware;
pub mod openapi;
pub mod user;
pub mod git;

pub fn openapi_router() -> Router<AppState> {
  let user_routes = user_routes();
  let auth_routes = auth_routes();
  let git_routes = git::git_routes();
  let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
    .nest("/api/v1/user", user_routes)
    .nest("/api/v1/auth", auth_routes)
    // mount git endpoints at root to avoid interfering with api namespace
    .nest("/git", git_routes.into())
    .route("/api-docs/openapi.json", get(openapi))
    .split_for_parts();
  // Swagger UI is disabled to avoid download during build
  router.clone()
}

pub fn build_router(app_state: AppState) -> Router<()> {
  let base_router = openapi_router();
  let router_with_middleware = apply_request_id_middleware(base_router.clone());
  let router = router_with_middleware.with_state(app_state);
  router
}
