#[cfg(all(feature = "api-docs", any(gity_release, not(debug_assertions))))]
compile_error!(
  "api-docs is for development builds only; release artifacts must not expose API documentation"
);

pub mod access;
mod api_application;
pub mod application;
pub mod auth;
pub mod ci;
mod code_index;
mod composition;
pub mod deployment_api;
pub mod events;
pub mod features;
pub mod git_http;
pub mod global_search;
pub mod lifecycle;
pub mod maintenance;
pub mod oauth;
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
mod users_tokens;
mod web_assets;
pub use composition::compose;
#[cfg(test)]
mod auth_tests;

use axum::Router;
use axum::http::{HeaderValue, header};
use gity_core::{Db, Settings};
use gity_git::GitService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
  pub db: Db,
  pub git: GitService,
  pub git_cluster: Option<gity_git_cluster::GitClusterModule>,
  pub search: gity_search::SearchIndex,
  pub global_search: fluxdi::Shared<global_search::GlobalSearchRuntime>,
  pub code_index: gity_code_intel::Store,
  pub settings: Arc<Settings>,
  pub storage: gity_core::Storage,
  pub identity: fluxdi::Shared<gity_identity::IdentityRepository>,
  pub oauth: fluxdi::Shared<gity_identity::OAuthService>,
  pub collaboration: fluxdi::Shared<gity_collaboration::CollaborationService>,
  pub ci: fluxdi::Shared<gity_ci::CiService>,
  pub projects: fluxdi::Shared<gity_projects::ProjectService>,
  pub repository_access: fluxdi::Shared<gity_projects::RepositoryAccessService>,
  #[cfg(feature = "packages")]
  pub artifacts: fluxdi::Shared<gity_artifacts::ArtifactService>,
}

pub fn router(state: AppState) -> Router {
  router_with_apis(state, Router::new())
}
pub(crate) fn router_with_apis(state: AppState, adapters: Router) -> Router {
  let api = Router::new()
    .merge(deployment_api::documented_routes().split_for_parts().0)
    .merge(auth::routes())
    .merge(users::routes())
    .merge(projects::routes())
    .merge(repository::routes())
    .merge(features::routes())
    .merge(global_search::documented_routes().split_for_parts().0)
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
    .merge(if state.settings.api.mcp_enabled {
      oauth::routes()
    } else {
      Router::new()
    })
    .merge(registry::lfs_protocol_routes().split_for_parts().0)
    .nest("/api/v1", api)
    .merge(git_http::routes())
    .merge(adapters.with_state::<AppState>(()))
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
  if request.uri().path() == "/mcp"
    || request.uri().path().starts_with("/.well-known/oauth-")
    || request.uri().path().starts_with("/oauth/") && request.uri().path() != "/oauth/consent"
    || request.uri().path() == "/openapi.json"
    || request.uri().path() == "/docs"
    || request.uri().path().starts_with("/docs/")
    || request.uri().path().starts_with("/swagger-ui")
  {
    return Err(gity_core::AppError::not_found("endpoint not found"));
  }
  if registry::is_lfs_path(request.uri().path()) {
    return registry::handle_lfs(state, request).await;
  }
  if request.uri().path().starts_with("/api/") || request.uri().path().contains(".git/") {
    return Err(gity_core::AppError::not_found("endpoint not found"));
  }
  let path = request.uri().path().to_owned();
  let brotli = accepts_brotli(request.headers());
  let relative = path.trim_start_matches('/');
  if relative.split('/').any(|segment| segment == "..") {
    return Err(gity_core::AppError::not_found("asset not found"));
  }
  let asset = web_assets::resolve(relative, brotli).or_else(|| {
    web_assets::is_spa_route(relative)
      .then(|| web_assets::resolve("index.html", brotli))
      .flatten()
  });
  let Some(asset) = asset else {
    return Err(gity_core::AppError::not_found("asset not found"));
  };
  let mut response = axum::response::Response::new(axum::body::Body::from(asset.body));
  let headers = response.headers_mut();
  headers.insert(
    header::CONTENT_TYPE,
    HeaderValue::from_static(asset.content_type),
  );
  if let Some(encoding) = asset.encoding
    && let Ok(value) = HeaderValue::from_str(encoding)
  {
    headers.insert(header::CONTENT_ENCODING, value);
  }
  if let Ok(value) = HeaderValue::from_str(cache_control(&path)) {
    headers.insert(header::CACHE_CONTROL, value);
  }
  vary_on_encoding(headers);
  Ok(response)
}

fn accepts_brotli(headers: &axum::http::HeaderMap) -> bool {
  let Some(value) = headers
    .get(header::ACCEPT_ENCODING)
    .and_then(|value| value.to_str().ok())
  else {
    return false;
  };
  value.split(',').any(|entry| {
    let mut parts = entry.split(';');
    let name = parts.next().unwrap_or_default().trim();
    let refused = parts.any(|parameter| {
      parameter
        .trim()
        .strip_prefix("q=")
        .is_some_and(|quality| quality.trim().parse::<f32>().is_ok_and(|q| q <= 0.0))
    });
    name.eq_ignore_ascii_case("br") && !refused
  })
}

/// Serving `<file>.br` itself does not advertise the encoding choice, so any
/// shared cache would key the plain and compressed variants under one entry.
fn vary_on_encoding(headers: &mut axum::http::HeaderMap) {
  let existing = headers
    .get(header::VARY)
    .and_then(|value| value.to_str().ok())
    .unwrap_or_default();
  if existing.to_ascii_lowercase().contains("accept-encoding") {
    return;
  }
  let combined = if existing.is_empty() {
    "Accept-Encoding".to_owned()
  } else {
    format!("{existing}, Accept-Encoding")
  };
  if let Ok(value) = HeaderValue::from_str(&combined) {
    headers.insert(header::VARY, value);
  }
}

/// Hashed assets never change under one URL; entries and client routes must revalidate.
fn cache_control(path: &str) -> &'static str {
  const IMMUTABLE: &str = "public, max-age=31536000, immutable";
  if path == "/" || !path.contains('.') {
    return "no-cache";
  }
  if hashed_asset(path) || path.ends_with(".wasm") {
    return IMMUTABLE;
  }
  "public, max-age=600"
}

/// Vite writes `assets/<name>-<8 char hash>.<ext>`, Trunk `<name>-<16 hex>.ext`.
fn hashed_asset(path: &str) -> bool {
  let file = path.rsplit('/').next().unwrap_or(path);
  let Some((stem, _)) = file.rsplit_once('.') else {
    return false;
  };
  let Some((_, hash)) = stem.rsplit_once('-') else {
    return false;
  };
  if path.starts_with("/assets/") && is_vite_hash(hash) {
    return true;
  }
  hash.len() == 16 && hash.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_vite_hash(hash: &str) -> bool {
  hash.len() == 8
    && hash
      .bytes()
      .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}
