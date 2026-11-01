mod package_binary;
mod package_cargo;
mod package_cargo_publish;
mod package_go;
mod package_lookup;
mod package_maven;
mod package_metadata;
mod package_npm_tags;
mod package_nuget;
mod package_nuget_registration;
mod package_nuget_search;
mod package_protocols;
mod package_pypi;
mod package_pypi_index;
mod package_storage;
mod packages;

pub use crate::openapi::package_schemas;
use crate::{PackageAction, PackageActor, PackageState};
use axum::{
  Json, Router,
  http::{HeaderMap, HeaderValue, StatusCode, header},
  middleware::Next,
  response::Response,
};
use gity_core::{AppError, externalize};
use serde_json::{Value, json};

type AppState = PackageState;
type Reply = Result<Json<Value>, AppError>;

pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<PackageState> {
  packages::documented_routes()
}

pub fn routes() -> Router<PackageState> {
  let router = documented_routes().split_for_parts().0;
  router
    .route(
      "/projects/{id}/packages/pypi/simple/",
      axum::routing::get(package_pypi_index::simple_index),
    )
    .route(
      "/projects/{id}/packages/pypi/simple/{name}/",
      axum::routing::get(package_pypi_index::simple_package),
    )
}

fn reply(value: Value) -> Reply {
  Ok(gity_core::ok(externalize(value)))
}

fn num(value: &Value, key: &str) -> i64 {
  value[key]
    .as_i64()
    .or_else(|| value[key].as_str().and_then(|value| value.parse().ok()))
    .unwrap_or(0)
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
  value[key].as_str().unwrap_or("")
}

fn required(value: &Value, key: &str) -> Result<String, AppError> {
  let text = text(value, key).trim();
  if text.is_empty() {
    Err(AppError::bad_request(format!("{key} is required")))
  } else {
    Ok(text.to_owned())
  }
}

fn request_origin(headers: &HeaderMap) -> Result<String, AppError> {
  let scheme = headers
    .get("x-forwarded-proto")
    .and_then(|value| value.to_str().ok())
    .filter(|value| matches!(*value, "http" | "https"))
    .unwrap_or("http");
  let host = headers
    .get("x-forwarded-host")
    .or_else(|| headers.get(header::HOST))
    .and_then(|value| value.to_str().ok())
    .unwrap_or("localhost");
  let uri = axum::http::Uri::builder()
    .scheme(scheme)
    .authority(host)
    .path_and_query("/")
    .build()
    .map_err(|_| AppError::bad_request("invalid request host"))?;
  Ok(format!("{}://{}", scheme, uri.authority().unwrap()))
}

async fn authorize(
  state: &AppState,
  headers: &HeaderMap,
  project_id: i64,
  action: PackageAction,
) -> Result<PackageActor, AppError> {
  state
    .authorization
    .authorize(project_id, action, headers)
    .await
}

async fn authorize_package_write(
  state: &AppState,
  headers: &HeaderMap,
  project_id: i64,
) -> Result<PackageActor, AppError> {
  authorize(state, headers, project_id, PackageAction::Write).await
}

async fn authorize_package_delete(
  state: &AppState,
  headers: &HeaderMap,
  project_id: i64,
) -> Result<PackageActor, AppError> {
  authorize(state, headers, project_id, PackageAction::Delete).await
}

async fn basic_challenge(request: axum::extract::Request, next: Next) -> Response {
  let mut response = next.run(request).await;
  if response.status() == StatusCode::UNAUTHORIZED {
    response.headers_mut().insert(
      header::WWW_AUTHENTICATE,
      HeaderValue::from_static("Basic realm=\"Gity package registry\""),
    );
  }
  response
}

fn timestamp(value: &str) -> Option<chrono::DateTime<chrono::Utc>> {
  chrono::DateTime::parse_from_rfc3339(value)
    .ok()
    .map(|value| value.to_utc())
    .or_else(|| {
      ["%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S%.f"]
        .iter()
        .find_map(|format| {
          chrono::NaiveDateTime::parse_from_str(value, format)
            .ok()
            .map(|value| value.and_utc())
        })
    })
}
