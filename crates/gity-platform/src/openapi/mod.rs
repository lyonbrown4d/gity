//! OpenAPI generated from the same annotated handlers used to register Axum routes.
pub mod activity_schemas;
pub mod credential_schemas;
pub mod discussion_schemas;
pub mod git_schemas;
pub mod code_schemas;
pub mod repository_schemas;
mod system;
pub use system::routes as system_routes;
pub mod issue_schemas;
pub mod label_schemas;
pub mod lfs_schemas;
pub mod merge_schemas;
pub mod milestone_schemas;
#[cfg(feature = "packages")]
pub use gity_package_registry::package_schemas;
pub mod project_schemas;
pub mod release_schemas;
pub mod schemas;
pub mod subscription_schemas;
pub mod user_schemas;
pub mod wiki_schemas;
use crate::AppState;
use axum::Router;
use utoipa::OpenApi;
#[derive(OpenApi)]
#[openapi(
  components(schemas(schemas::ErrorEnvelope)),
  info(
    title = "Gity API",
    version = "0.2.0",
    description = "Rust collaboration, Git hosting, package registry and CI APIs."
  )
)]
struct BaseApi;
pub fn document() -> utoipa::openapi::OpenApi {
  let mut api = BaseApi::openapi();
  api = api.nest(
    "/api/v1",
    crate::auth::documented_routes().split_for_parts().1,
  );
  api = api.nest(
    "/api/v1",
    crate::users::documented_routes().split_for_parts().1,
  );
  api = api.nest(
    "/api/v1",
    crate::projects::documented_routes().split_for_parts().1,
  );
  api = api.nest(
    "/api/v1",
    crate::features::documented_routes().split_for_parts().1,
  );
  api = api.nest(
    "/api/v1",
    crate::registry::documented_routes().split_for_parts().1,
  );
  api = api.nest(
    "/api/v1",
    crate::ci::documented_routes().split_for_parts().1,
  );
  api = api.nest(
    "/api/v1",
    crate::repository::documented_routes().split_for_parts().1,
  );
  #[cfg(feature = "packages")]
  {
    api = api.nest(
      "/api/v1",
      gity_package_registry::documented_routes()
        .split_for_parts()
        .1,
    );
  }
  api.merge(crate::git_http::documented_routes().split_for_parts().1);
  api.merge(system::routes().split_for_parts().1);
  api.merge(crate::oauth::documented_routes().split_for_parts().1);
  api.merge(crate::registry::lfs_protocol_routes().split_for_parts().1);
  api = api.nest(
    "/api/v1",
    crate::deployment_api::documented_routes()
      .split_for_parts()
      .1,
  );
  #[cfg(feature = "oci")]
  {
    api = api.nest(
      "/api/v1",
      crate::oci_registry::documented_routes().split_for_parts().1,
    );
  }
  normalize(&mut api);
  let components = api.components.get_or_insert_default();
  use utoipa::openapi::security::{ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme};
  components.add_security_scheme(
    "bearer",
    SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
  );
  components.add_security_scheme(
    "basic",
    SecurityScheme::Http(Http::new(HttpAuthScheme::Basic)),
  );
  components.add_security_scheme(
    "private_token",
    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("PRIVATE-TOKEN"))),
  );
  api
}
pub fn routes() -> Router<AppState> {
  #[cfg(feature = "api-docs")]
  return Router::from(utoipa_swagger_ui::SwaggerUi::new("/docs").url("/openapi.json", document()));
  #[cfg(not(feature = "api-docs"))]
  Router::new()
}
/// Legacy aliases reuse their canonical operation and schema.
pub fn alias(
  router: &mut utoipa_axum::router::OpenApiRouter<AppState>,
  source: &str,
  target: &str,
) {
  let item = router
    .get_openapi()
    .paths
    .paths
    .get(source)
    .expect("documented alias source must be registered")
    .clone();
  router
    .get_openapi_mut()
    .paths
    .paths
    .insert(target.into(), item);
}

fn normalize(api: &mut utoipa::openapi::OpenApi) {
  let paths = std::mem::take(&mut api.paths.paths);
  for (path, mut item) in paths {
    let path = path.replace("{*", "{");
    for (method, operation) in [
      ("get", &mut item.get),
      ("post", &mut item.post),
      ("put", &mut item.put),
      ("patch", &mut item.patch),
      ("delete", &mut item.delete),
      ("head", &mut item.head),
    ] {
      if let Some(operation) = operation {
        let suffix = path
          .chars()
          .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
          .collect::<String>();
        operation.operation_id = Some(format!("{method}_{suffix}"));
        if path.starts_with("/api/v1/repos/") {
          operation.deprecated = Some(utoipa::openapi::Deprecated::True);
        }
      }
    }
    api.paths.paths.insert(path, item);
  }
}
#[cfg(test)]
mod legacy_contract_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod view_tests;

/// Serializes [`document`] as pretty JSON into `path`, creating parent directories.
pub fn write_document(path: &std::path::Path) -> anyhow::Result<()> {
  if let Some(parent) = path
    .parent()
    .filter(|parent| !parent.as_os_str().is_empty())
  {
    std::fs::create_dir_all(parent)?;
  }
  let json = serde_json::to_string_pretty(&document())?;
  std::fs::write(path, format!("{json}\n"))?;
  Ok(())
}
