use crate::app_state::AppState;
use crate::rest::auth::__path_login;
use crate::rest::auth::login;
use crate::rest::openapi::{ApiDoc, openapi};

use crate::rest::middleware::apply_request_id_middleware;
use crate::rest::user::user_routes;
use axum::Router;
use axum::routing::get;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

pub mod auth;
pub mod middleware;
pub mod openapi;
pub mod user;

pub fn openapi_router() -> Router<AppState> {
  let user_routes = user_routes();
  let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
    .nest("/api/v1/user", user_routes)
    .routes(routes![login])
    .route("/api-docs/openapi.json", get(openapi))
    .split_for_parts();
  router
    .merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api))
    .clone()
}

pub fn build_router(app_state: AppState) -> Router<()> {
  let base_router = openapi_router();
  let router_with_middleware = apply_request_id_middleware(base_router.clone());
  let router = router_with_middleware.with_state(app_state);
  router
}
