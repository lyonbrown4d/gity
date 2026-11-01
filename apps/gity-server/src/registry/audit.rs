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
    reply(json!(s.db.all("SELECT * FROM project_audit_events WHERE project_id = ? ORDER BY created_at DESC,id DESC LIMIT ?",vec![p.into(),limit.into()]).await?))
}
/// Persist the existing domain event audit shape. Callers must omit secrets from payload.
pub struct AuditRecord<'a> {
    pub project: i64,
    pub actor: i64,
    pub event: &'a str,
    pub action: &'a str,
    pub target_type: &'a str,
    pub target_id: &'a str,
    pub summary: &'a str,
    pub payload: Value,
}
pub async fn record_event(s: &AppState, record: AuditRecord<'_>) -> Result<(), AppError> {
    let AuditRecord {
        project,
        actor,
        event,
        action,
        target_type,
        target_id,
        summary,
        payload,
    } = record;
    let org =
        s.db.one(
            "SELECT organization_id FROM projects WHERE id = ?",
            vec![project.into()],
        )
        .await?
        .ok_or_else(|| AppError::not_found("project not found"))?;
    let sql = concat!(
        "INSERT INTO project_audit_events(id,project_id,organization_id,event_name,action,",
        "actor_user_id,target_type,target_id,summary,payload,created_at) VALUES (?,?,?,?,?,?,?,?,?,?,?)"
    );
    let args = vec![
        gity_core::new_id().into(),
        project.into(),
        num(&org, "organization_id").into(),
        event.into(),
        action.into(),
        actor.into(),
        target_type.into(),
        target_id.into(),
        summary.into(),
        payload.to_string().into(),
        now().into(),
    ];
    s.db.execute(sql, args).await?;
    Ok(())
}
