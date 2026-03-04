use crate::http::app_state::AppState;
use crate::http::middleware::apply_request_id_middleware;
use crate::http::openapi::{ApiDoc, openapi};
use axum::Router;
use axum::routing::get;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;
// Swagger UI removed to avoid build-time downloads

pub mod app_state;
pub mod auth;
pub mod git;
pub mod middleware;
pub mod openapi;
pub mod organization;
pub mod repo;
pub mod user;

pub fn openapi_router() -> Router<AppState> {
  let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
    .nest("/api/v1/auth", auth::auth_routes())
    .nest("/api/v1/orgs", organization::organization_routes())
    .nest("/api/v1/repos", repo::repo_routes())
    .nest("/api/v1/users", user::user_routes())
    .nest("/git", git::git_routes().into())
    .route("/api-docs/openapi.json", get(openapi))
    .split_for_parts();
  // Swagger UI is disabled to avoid download during build
  router
    .merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api))
    .clone()
}
//
pub fn build_router(app_state: AppState) -> Router<()> {
  let cors_allowed_origins = app_state.config.server.cors_allowed_origins.clone();
  let base_router = openapi_router();
  let router_with_middleware =
    apply_request_id_middleware(base_router.clone(), cors_allowed_origins);
  let router = router_with_middleware.with_state(app_state);
  router
}
