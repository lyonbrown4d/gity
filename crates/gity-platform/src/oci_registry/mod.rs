//! OCI authorization shares the main application's identity and project authority.
mod authorizer;
mod policies;
pub use authorizer::PlatformOciAuthorizer;
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<crate::AppState> {
  policies::documented_routes()
}
