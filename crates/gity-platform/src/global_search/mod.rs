mod hydrate;
mod projection;
mod runtime;
mod schemas;
mod scope;
mod service;
mod session;
pub(crate) mod worker;
use crate::{AppState, openapi::schemas as api};
use axum::{Json, extract::State, http::HeaderMap};
pub use runtime::GlobalSearchRuntime;
pub(crate) use schemas::*;

pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!(search))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(post,path="/search",tag="Search",request_body=SearchRequest,
  responses((status=200,description="Permission-scoped global code and issue search",body=api::Envelope<SearchReply>),api::Errors),
  security(("bearer"=[])))]
async fn search(
  State(state): State<AppState>,
  headers: HeaderMap,
  Json(input): Json<SearchRequest>,
) -> Result<Json<serde_json::Value>, gity_core::AppError> {
  let reply = tokio::time::timeout(
    std::time::Duration::from_secs(30),
    service::search(&state, &headers, input),
  )
  .await
  .map_err(|_| {
    gity_core::AppError::new(
      axum::http::StatusCode::SERVICE_UNAVAILABLE,
      "search request timed out",
    )
  })??;
  Ok(gity_core::ok(serde_json::to_value(reply).map_err(
    |_| gity_core::AppError::internal("cannot encode search result"),
  )?))
}

#[cfg(test)]
mod cursor_tests;
#[cfg(test)]
mod tests;
