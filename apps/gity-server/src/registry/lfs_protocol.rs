//! Protocol entrypoints share the same request handler as the legacy namespace fallback.
use super::*;
use crate::openapi::{lfs_schemas as spec, schemas as api};
use axum::{
    extract::{Request, State},
    response::Response,
};
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
    utoipa_axum::router::OpenApiRouter::new()
        .routes(utoipa_axum::routes!(batch))
        .routes(utoipa_axum::routes!(download, upload))
        .routes(utoipa_axum::routes!(locks, create_lock))
        .routes(utoipa_axum::routes!(verify))
        .routes(utoipa_axum::routes!(unlock))
        .layer(axum::middleware::from_fn(crate::auth::basic_challenge))
}
#[utoipa::path(
    post,path="/{organization}/{project}/info/lfs/objects/batch",tag="Git LFS protocol",
    params(spec::RepositoryPath),request_body=spec::BatchRequest,
    responses((status=200,description="Basic-transfer object actions or per-object errors",body=spec::BatchResponse,content_type="application/vnd.git-lfs+json"),api::Errors),
    security((),("bearer"=[]),("basic"=[]))
)]
async fn batch(State(s): State<AppState>, request: Request) -> Result<Response, AppError> {
    handle_lfs(s, request).await
}
#[utoipa::path(
    get,path="/{organization}/{project}/info/lfs/objects/{oid}",tag="Git LFS protocol",
    params(spec::ObjectPath),
    responses((status=200,description="Verified LFS object bytes",body=api::Binary,content_type="application/octet-stream"),api::Errors),
    security((),("bearer"=[]),("basic"=[]))
)]
async fn download(State(s): State<AppState>, request: Request) -> Result<Response, AppError> {
    handle_lfs(s, request).await
}
#[utoipa::path(
    put,path="/{organization}/{project}/info/lfs/objects/{oid}",tag="Git LFS protocol",
    params(spec::ObjectPath),request_body(content=api::Binary,content_type="application/octet-stream"),
    responses((status=200,description="Object uploaded after SHA256 verification"),api::Errors),
    security(("bearer"=[]),("basic"=[]))
)]
async fn upload(State(s): State<AppState>, request: Request) -> Result<Response, AppError> {
    handle_lfs(s, request).await
}
#[utoipa::path(
    get,path="/{organization}/{project}/info/lfs/locks",tag="Git LFS protocol",
    params(spec::RepositoryPath,("path"=Option<String>,Query),("id"=Option<String>,Query),("cursor"=Option<String>,Query),("limit"=Option<i64>,Query)),
    responses((status=200,description="Project-scoped lock page",body=spec::LfsLocks),api::Errors),
    security((),("bearer"=[]),("basic"=[]))
)]
async fn locks(State(s): State<AppState>, request: Request) -> Result<Response, AppError> {
    handle_lfs(s, request).await
}
#[utoipa::path(
    post,path="/{organization}/{project}/info/lfs/locks",tag="Git LFS protocol",
    params(spec::RepositoryPath),request_body=spec::CreateLock,
    responses((status=201,description="Lock created",body=spec::LockEnvelope),api::Errors),
    security(("bearer"=[]),("basic"=[]))
)]
async fn create_lock(State(s): State<AppState>, request: Request) -> Result<Response, AppError> {
    handle_lfs(s, request).await
}
#[utoipa::path(
    post,path="/{organization}/{project}/info/lfs/locks/verify",tag="Git LFS protocol",
    params(spec::RepositoryPath),request_body=spec::VerifyLocks,
    responses((status=200,description="Locks separated by authenticated ownership",body=spec::VerifiedLocks),api::Errors),
    security(("bearer"=[]),("basic"=[]))
)]
async fn verify(State(s): State<AppState>, request: Request) -> Result<Response, AppError> {
    handle_lfs(s, request).await
}
#[utoipa::path(
    post,path="/{organization}/{project}/info/lfs/locks/{lock}/unlock",tag="Git LFS protocol",
    params(spec::UnlockPath),request_body=spec::Unlock,
    responses((status=200,description="Lock released; force requires project write access",body=spec::LockEnvelope),api::Errors),
    security(("bearer"=[]),("basic"=[]))
)]
async fn unlock(State(s): State<AppState>, request: Request) -> Result<Response, AppError> {
    handle_lfs(s, request).await
}
