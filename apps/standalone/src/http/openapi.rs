use axum::Json;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
  paths(openapi),
  tags(
    (name = "System", description = "System and documentation endpoints"),
    (name = "Auth", description = "Authentication and token flows"),
    (name = "Users", description = "User profile and admin user management"),
    (name = "Organizations", description = "Organization and membership management"),
    (name = "Repositories", description = "Repository, source, issue, and branch operations")
  )
)]
pub struct ApiDoc;

/// Return JSON version of an OpenAPI schema
#[utoipa::path(
    get,
    path = "/api-docs/openapi.json",
    tag = "System",
    responses(
        (status = 200, description = "JSON file", body = ())
    )
)]
pub async fn openapi() -> Json<utoipa::openapi::OpenApi> {
  Json(ApiDoc::openapi())
}
