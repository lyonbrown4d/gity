use super::schemas::Envelope;
use crate::AppState;
use axum::Json;
use gity_core::ok;
use serde::Serialize;
use serde_json::{Value, json};
use utoipa::ToSchema;
#[derive(Serialize, ToSchema)]
pub struct Health {
  pub status: String,
  pub runtime: String,
}
#[derive(Serialize, ToSchema)]
pub struct Runtime {
  pub backend: String,
  pub frontend: String,
  pub version: String,
}
pub fn routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(health))
    .routes(utoipa_axum::routes!(runtime))
}
#[utoipa::path(get,path="/api/health",tag="System",responses((status=200,description="Server process health",body=Envelope<Health>)),security())]
async fn health() -> Json<Value> {
  ok(json!({"status":"ok","runtime":"rust"}))
}
#[utoipa::path(get,path="/api/v1/rewrite/info",tag="System",responses((status=200,description="Runtime and build version",body=Envelope<Runtime>)),security())]
async fn runtime() -> Json<Value> {
  ok(json!({"backend":"rust","frontend":"leptos","version":env!("CARGO_PKG_VERSION")}))
}
