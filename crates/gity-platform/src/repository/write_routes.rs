use super::*;
use crate::openapi::{repository_schemas as spec, schemas as api};
pub(super) fn routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(branches))
    .routes(utoipa_axum::routes!(files))
    .routes(utoipa_axum::routes!(file_commits))
    .routes(utoipa_axum::routes!(delete_branch))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(post,path="/branches",tag="Repository",params(api::ProjectPath),request_body=spec::CreateBranch,
    responses((status=200,description="Create branch; requires developer and matching protection permission",body=api::Envelope<spec::CreatedBranch>),api::Errors),security(("bearer"=[]),("private_token"=[])))]
pub(super) async fn branches(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Json(body): Json<Value>,
) -> Reply {
  write::write(State(s), h, Path((id, "branches".into())), Json(body)).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(post,path="/files",tag="Repository",params(api::ProjectPath),request_body=spec::FileCommitInput,
    responses((status=200,description="Commit a file with authenticated author; triggers repository events and CI",body=api::Envelope<spec::FileCommit>),api::Errors),security(("bearer"=[]),("private_token"=[])))]
pub(super) async fn files(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Json(body): Json<Value>,
) -> Reply {
  write::write(State(s), h, Path((id, "files".into())), Json(body)).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(post,path="/file-commits",tag="Repository",params(api::ProjectPath),request_body=spec::FileCommitInput,
    responses((status=200,description="Commit a file with authenticated author; triggers repository events and CI",body=api::Envelope<spec::FileCommit>),api::Errors),security(("bearer"=[]),("private_token"=[])))]
pub(super) async fn file_commits(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Json(body): Json<Value>,
) -> Reply {
  write::write(State(s), h, Path((id, "file-commits".into())), Json(body)).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(delete,path="/branches/{branch}",tag="Repository",params(spec::BranchPath),
    responses((status=200,description="Branch deleted; maintainer required, default branch cannot be deleted",body=api::Envelope<spec::Ack>),api::Errors),security(("bearer"=[]),("private_token"=[])))]
async fn delete_branch(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(path): Path<(i64, String)>,
) -> Reply {
  write::delete_branch(State(s), h, Path(path)).await
}
