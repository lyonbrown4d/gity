use crate::openapi::{git_schemas as spec, schemas as api};
mod rpc;
use crate::{AppState, access, repository};
use axum::{
  Router,
  body::Body,
  extract::{Path, Query, Request, State},
  http::{HeaderMap, StatusCode, header},
  response::{IntoResponse, Response},
};
use base64::Engine;
use gity_core::AppError;
use gity_git::GitRpc;
use rpc::rpc;
use std::collections::{BTreeMap, HashMap};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::io::{ReaderStream, StreamReader};

pub fn routes() -> Router<AppState> {
  documented_routes().split_for_parts().0
}
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(advertise))
    .routes(utoipa_axum::routes!(upload))
    .routes(utoipa_axum::routes!(receive))
    .layer(axum::middleware::from_fn(crate::auth::basic_challenge))
}
async fn project(
  s: &AppState,
  headers: &HeaderMap,
  org: &str,
  name: &str,
  write: bool,
) -> Result<(serde_json::Value, HeaderMap), AppError> {
  let name = name
    .strip_suffix(".git")
    .ok_or_else(|| AppError::not_found("repository not found"))?;
  let project = s
    .db
    .one(
      "SELECT * FROM projects WHERE full_path=? AND deleted_at IS NULL",
      vec![format!("{org}/{name}").into()],
    )
    .await?
    .ok_or_else(|| AppError::not_found("repository not found"))?;
  let mut h = headers.clone();
  if let Some(value) = headers
    .get(header::AUTHORIZATION)
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.strip_prefix("Basic "))
  {
    let decoded = base64::engine::general_purpose::STANDARD
      .decode(value)
      .map_err(|_| AppError::unauthorized("invalid credentials"))?;
    let text =
      std::str::from_utf8(&decoded).map_err(|_| AppError::unauthorized("invalid credentials"))?;
    let (_, token) = text
      .split_once(':')
      .ok_or_else(|| AppError::unauthorized("token required"))?;
    h.insert(
      header::AUTHORIZATION,
      format!("Bearer {token}")
        .parse()
        .map_err(|_| AppError::unauthorized("invalid credentials"))?,
    );
  }
  if let Some(raw) = crate::registry::token_from_headers(&h)
    && let Some(token) = crate::registry::authenticate_token(s, &raw).await?
  {
    let allowed = token.project_id == crate::projects::id(&project["id"])?
      && (token.allows("write_repository")
        || !write
          && ["read_repository", "read_api", "write_api"]
            .iter()
            .any(|scope| token.allows(scope)));
    if !allowed {
      return Err(AppError::forbidden(
        "project token does not permit this repository operation",
      ));
    }
    return Ok((project, h));
  }
  access::project(
    s,
    &h,
    crate::projects::id(&project["id"])?,
    if write { 30 } else { 0 },
  )
  .await?;
  Ok((project, h))
}
#[utoipa::path(get,path="/{organization}/{project}/info/refs",tag="Git Smart HTTP",params(spec::GitPath,spec::DiscoveryQuery,
    ("Git-Protocol"=Option<String>,Header,description="Optional Git protocol negotiation, such as version=2")),
    responses((status=200,description="Git service discovery packet lines",content((api::Binary="application/x-git-upload-pack-advertisement"),(api::Binary="application/x-git-receive-pack-advertisement"))),api::Errors),security((),("basic"=[]),("bearer"=[]),("private_token"=[])))]
async fn advertise(
  State(s): State<AppState>,
  Path((org, name)): Path<(String, String)>,
  Query(q): Query<HashMap<String, String>>,
  h: HeaderMap,
) -> Result<Response, AppError> {
  let service = q.get("service").map(String::as_str).unwrap_or_default();
  let rpc = match service {
    "git-upload-pack" => GitRpc::UploadPack,
    "git-receive-pack" => GitRpc::ReceivePack,
    _ => return Err(AppError::bad_request("unsupported Git service")),
  };
  let (p, _) = project(&s, &h, &org, &name, rpc == GitRpc::ReceivePack).await?;
  let mut env = BTreeMap::new();
  if let Some(v) = h.get("git-protocol").and_then(|v| v.to_str().ok()) {
    env.insert("GIT_PROTOCOL".into(), v.into());
  }
  if rpc == GitRpc::ReceivePack {
    s.git
      .ensure_update_hook(
        &repository::repo(&p),
        s.settings
          .git_hook
          .clone()
          .unwrap_or(std::env::current_exe()?),
      )
      .await
      .map_err(repository::git_error)?;
  }
  let mut process = s
    .git
    .spawn_rpc(&repository::repo(&p), rpc, true, env)
    .await
    .map_err(repository::git_error)?;
  drop(process.take_stdin());
  // Discovery output is small and bounded separately from streaming pack bodies.
  let reader = process.into_reader().map_err(repository::git_error)?;
  let mut bytes = Vec::new();
  reader.take(8 * 1024 * 1024).read_to_end(&mut bytes).await?;
  let prefix = format!("# service={service}\n");
  let mut body = format!("{:04x}{}0000", prefix.len() + 4, prefix).into_bytes();
  body.extend(bytes);
  Ok(
    (
      [
        (
          header::CONTENT_TYPE,
          format!("application/x-{service}-advertisement"),
        ),
        (header::CACHE_CONTROL, "no-cache".into()),
      ],
      body,
    )
      .into_response(),
  )
}
#[utoipa::path(post,path="/{organization}/{project}/git-upload-pack",tag="Git Smart HTTP",params(spec::GitPath,
    ("Git-Protocol"=Option<String>,Header,description="Optional Git protocol negotiation")),
    request_body(content=api::Binary,content_type="application/x-git-upload-pack-request"),
    responses((status=200,description="Stream Git fetch response after project read authorization",body=api::Binary,content_type="application/x-git-upload-pack-result"),api::Errors),security((),("basic"=[]),("bearer"=[]),("private_token"=[])))]
async fn upload(
  State(s): State<AppState>,
  Path((org, name)): Path<(String, String)>,
  req: Request,
) -> Result<Response, AppError> {
  rpc(s, org, name, req, false).await
}
#[utoipa::path(post,path="/{organization}/{project}/git-receive-pack",tag="Git Smart HTTP",params(spec::GitPath,
    ("Git-Protocol"=Option<String>,Header,description="Optional Git protocol negotiation")),
    request_body(content=api::Binary,content_type="application/x-git-receive-pack-request"),
    responses((status=200,description="Stream Git push response; developer or scoped write_repository token; hooks enforce branch protection",body=api::Binary,content_type="application/x-git-receive-pack-result"),api::Errors),security(("basic"=[]),("bearer"=[]),("private_token"=[])))]
async fn receive(
  State(s): State<AppState>,
  Path((org, name)): Path<(String, String)>,
  req: Request,
) -> Result<Response, AppError> {
  rpc(s, org, name, req, true).await
}
