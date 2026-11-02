//! OCI authorization shares the main application's identity and project authority.
mod authorizer;
#[cfg(test)]
mod mount_tests;
mod policies;
#[cfg(test)]
mod tests;
pub use authorizer::PlatformOciAuthorizer;
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<crate::AppState> {
  policies::documented_routes()
}
