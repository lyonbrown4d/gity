use super::*;
use crate::openapi::{credential_schemas as spec, schemas as api};
use axum::{
  extract::{Path, Query, State},
  http::HeaderMap,
};
use std::collections::HashMap;
pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!(events))
}
#[utoipa::path(
    get,path="/projects/{id}/audit-events",tag="audit",
    params(api::ProjectPath,("limit"=Option<i64>,Query,description="1–200; defaults to 50")),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::AuditEvent>>),api::Errors),
    security(("bearer"=[]),("basic"=[]),("private_token"=[]))
)]
async fn events(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  crate::access::project_action(&s, &h, p, crate::access::ProjectAction::Write).await?;
  let limit = q
    .get("limit")
    .and_then(|v| v.parse::<i64>().ok())
    .filter(|v| *v > 0 && *v <= 200)
    .unwrap_or(50);
  reply(json!(
    s.repository_access.audit_events(p, limit as u64).await?
  ))
}
pub use gity_projects::AuditRecord;
pub async fn record_event(s: &AppState, record: AuditRecord<'_>) -> Result<(), AppError> {
  s.repository_access.record_event(record).await
}
