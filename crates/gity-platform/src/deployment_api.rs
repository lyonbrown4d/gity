//! Deployment controls expose identifiers and capabilities, never peer credentials.
use crate::{AppState, auth, openapi::schemas as api};
use axum::{
  Json,
  extract::{Path, State},
  http::HeaderMap,
};
use gity_core::AppError;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Capabilities {
  pub git: bool,
  pub ci: bool,
  pub package_registry: bool,
  pub container_registry: bool,
}
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Promote {
  pub node: String,
}
#[derive(Serialize, ToSchema)]
pub struct ReplicaView {
  pub node: String,
  pub state: String,
  pub generation: i64,
}
#[derive(Serialize, ToSchema)]
pub struct PlacementView {
  pub project_id: String,
  pub primary_node: String,
  pub epoch: i64,
  pub generation: i64,
  pub pending_writes: i64,
  pub state: String,
  pub transition_target: Option<String>,
  pub replicas: Vec<ReplicaView>,
}
fn view(row: gity_git_cluster::Placement) -> PlacementView {
  PlacementView {
    project_id: row.project_id.to_string(),
    primary_node: row.primary_node,
    epoch: row.epoch,
    generation: row.generation,
    pending_writes: row.pending_writes,
    state: row.state,
    transition_target: row.transition_target,
    replicas: row
      .replicas
      .into_iter()
      .map(|replica| ReplicaView {
        node: replica.node,
        state: format!("{:?}", replica.status).to_lowercase(),
        generation: replica.generation,
      })
      .collect(),
  }
}
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(capabilities))
    .routes(utoipa_axum::routes!(placement))
    .routes(utoipa_axum::routes!(promote))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(get,path="/capabilities",tag="system",responses((status=200,description="Enabled product capabilities",body=api::Envelope<Capabilities>)))]
async fn capabilities(State(state): State<AppState>) -> Json<Value> {
  gity_core::ok(json!(Capabilities {
    git: true,
    ci: true,
    package_registry: cfg!(feature = "packages"),
    container_registry: cfg!(feature = "oci")
      && (state.settings.components.standalone || state.settings.components.authority.enabled)
      && state.settings.components.registry.enabled
  }))
}
async fn administrator(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
  if !auth::principal(state, headers).await?.is_super_admin {
    return Err(AppError::forbidden("administrator permission required"));
  }
  Ok(())
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(get,path="/admin/git/repositories/{id}/placement",tag="Git cluster",params(api::ProjectPath),responses((status=200,description="Current fenced repository placement",body=api::Envelope<PlacementView>),api::Errors),security(("bearer"=[])))]
async fn placement(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path(project): Path<i64>,
) -> Result<Json<Value>, AppError> {
  administrator(&state, &headers).await?;
  let cluster = state
    .git_cluster
    .as_ref()
    .ok_or_else(|| AppError::bad_request("Git cluster is not configured"))?
    .catalog()
    .map_err(|_| AppError::internal("Git cluster configuration unavailable"))?;
  let row = cluster.inspect(project).await.map_err(|_| {
    AppError::new(
      axum::http::StatusCode::SERVICE_UNAVAILABLE,
      "Git placement unavailable",
    )
  })?;
  Ok(gity_core::ok(json!(view(row))))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(post,path="/admin/git/repositories/{id}/promote",tag="Git cluster",params(api::ProjectPath),request_body=Promote,responses((status=200,description="Fenced primary handover after final replica verification",body=api::Envelope<PlacementView>),api::Errors),security(("bearer"=[])))]
async fn promote(
  State(state): State<AppState>,
  headers: HeaderMap,
  Path(project): Path<i64>,
  Json(body): Json<Promote>,
) -> Result<Json<Value>, AppError> {
  administrator(&state, &headers).await?;
  state
    .identity
    .active_project(project)
    .await?
    .ok_or_else(|| AppError::not_found("project not found"))?;
  let cluster = state
    .git_cluster
    .as_ref()
    .ok_or_else(|| AppError::bad_request("Git cluster is not configured"))?
    .catalog()
    .map_err(|_| AppError::internal("Git cluster configuration unavailable"))?;
  let row = cluster
    .promote(project, &body.node)
    .await
    .map_err(|error| {
      tracing::warn!(project,error=%error,"controlled Git handover requires retry or recovery");
      AppError::new(
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        "Git handover incomplete; inspect placement before retrying the same target",
      )
    })?;
  Ok(gity_core::ok(json!(view(row))))
}
