use axum::Json;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
  paths(openapi),
  tags(
    (name = "System", description = "System and documentation endpoints"),
    (name = "Auth", description = "Authentication and token flows"),
    (name = "Users", description = "User profile and admin user management"),
    (name = "Organizations", description = "Legacy organization and membership management endpoints"),
    (name = "Repositories", description = "Legacy repository, source, issue, and branch endpoints"),
    (name = "Namespaces", description = "GitLab-like namespace management on the new project-centric model"),
    (name = "Projects", description = "GitLab-like project management on the new project-centric model")
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
