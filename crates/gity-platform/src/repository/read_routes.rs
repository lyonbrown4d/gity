use super::*;
use crate::openapi::{repository_schemas as spec, schemas as api};
pub(super) fn routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(branches))
    .routes(utoipa_axum::routes!(tree))
    .routes(utoipa_axum::routes!(blob))
    .routes(utoipa_axum::routes!(readme))
    .routes(utoipa_axum::routes!(languages))
    .routes(utoipa_axum::routes!(commits))
    .routes(utoipa_axum::routes!(search))
    .routes(utoipa_axum::routes!(branch_protections))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(get,path="/branches",tag="Repository",
    params(api::ProjectPath,spec::RepositoryQuery),
    responses((status=200,description="List branches with effective protection",body=api::Envelope<Vec<spec::Branch>>),api::Errors),
    security((),("bearer"=[]),("private_token"=[])))]
pub(super) async fn branches(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  read::read(State(s), h, Path((id, "branches".into())), Query(q)).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(get,path="/repos/{id}/tags",tag="Repository",
    params(api::ProjectPath,spec::RepositoryQuery),
    responses((status=200,description="List Git tags",body=api::Envelope<Vec<crate::openapi::release_schemas::Tag>>),api::Errors),
    security((),("bearer"=[]),("private_token"=[])))]
pub(super) async fn tags(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  read::read(State(s), h, Path((id, "tags".into())), Query(q)).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(get,path="/tree",tag="Repository",
    params(api::ProjectPath,spec::RepositoryQuery),
    responses((status=200,description="List files and directories at revision",body=api::Envelope<Vec<spec::TreeEntry>>),api::Errors),
    security((),("bearer"=[]),("private_token"=[])))]
pub(super) async fn tree(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  read::read(State(s), h, Path((id, "tree".into())), Query(q)).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(get,path="/blob",tag="Repository",
    params(api::ProjectPath,spec::RepositoryQuery),
    responses((status=200,description="Read UTF-8 or base64 file content",body=api::Envelope<spec::Blob>),api::Errors),
    security((),("bearer"=[]),("private_token"=[])))]
pub(super) async fn blob(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  read::read(State(s), h, Path((id, "blob".into())), Query(q)).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(get,path="/readme",tag="Repository",
    params(api::ProjectPath,spec::RepositoryQuery),
    responses((status=200,description="Read repository README",body=api::Envelope<spec::Blob>),api::Errors),
    security((),("bearer"=[]),("private_token"=[])))]
pub(super) async fn readme(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  read::read(State(s), h, Path((id, "readme".into())), Query(q)).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(get,path="/languages",tag="Repository",
    params(api::ProjectPath,spec::RepositoryQuery),
    responses((status=200,description="Analyze repository languages",body=api::Envelope<spec::Languages>),api::Errors),
    security((),("bearer"=[]),("private_token"=[])))]
pub(super) async fn languages(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  read::read(State(s), h, Path((id, "languages".into())), Query(q)).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(get,path="/commits",tag="Repository",
    params(api::ProjectPath,spec::RepositoryQuery),
    responses((status=200,description="List repository commits",body=api::Envelope<Vec<spec::Commit>>),api::Errors),
    security((),("bearer"=[]),("private_token"=[])))]
pub(super) async fn commits(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  read::read(State(s), h, Path((id, "commits".into())), Query(q)).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(get,path="/search",tag="Repository",
    params(api::ProjectPath,spec::RepositoryQuery),
    responses((status=200,description="Search text with optional case sensitivity and regular expressions",body=api::Envelope<Vec<spec::SearchMatch>>),api::Errors),
    security((),("bearer"=[]),("private_token"=[])))]
pub(super) async fn search(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  read::read(State(s), h, Path((id, "search".into())), Query(q)).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects/{id}/repository")]
#[utoipa::path(get,path="/branch-protections",tag="Repository",
    params(api::ProjectPath,spec::RepositoryQuery),
    responses((status=200,description="List branch protection rules",body=api::Envelope<Vec<spec::BranchProtection>>),api::Errors),
    security((),("bearer"=[]),("private_token"=[])))]
pub(super) async fn branch_protections(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  read::read(
    State(s),
    h,
    Path((id, "branch-protections".into())),
    Query(q),
  )
  .await
}
