use super::credential_views as views;
use super::tokens::active;
use super::*;
use crate::openapi::{credential_schemas as spec, schemas as api};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
};
pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(access_tokens, create_access))
    .routes(utoipa_axum::routes!(revoke_access))
    .routes(utoipa_axum::routes!(deploy_tokens, create_deploy))
    .routes(utoipa_axum::routes!(revoke_deploy))
    .routes(utoipa_axum::routes!(keys, create_key))
    .routes(utoipa_axum::routes!(delete_key))
}
fn token_view(mut row: Value) -> Value {
  let enabled = active(&row);
  row["scopes"] = json!(
    text(&row, "scopes")
      .split(',')
      .map(str::trim)
      .filter(|v| !v.is_empty())
      .collect::<Vec<_>>()
  );
  row["active"] = json!(enabled);
  row.as_object_mut().unwrap().remove("token");
  row
}
async fn list(s: AppState, h: HeaderMap, p: i64, kind: &str) -> Reply {
  crate::access::project_action(&s, &h, p, crate::access::ProjectAction::RepositoryAdmin).await?;
  reply(json!(
    s.repository_access
      .list_tokens(p, kind)
      .await?
      .iter()
      .map(views::token_json)
      .map(token_view)
      .collect::<Vec<_>>()
  ))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    get,path="/projects/{id}/access-tokens",tag="credentials",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::ProjectToken>>),api::Errors),
    security(("bearer"=[]))
)]
async fn access_tokens(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  list(s, h, p, "project_access").await
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    get,path="/projects/{id}/deploy-tokens",tag="credentials",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::ProjectToken>>),api::Errors),
    security(("bearer"=[]))
)]
async fn deploy_tokens(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  list(s, h, p, "deploy").await
}
async fn create(s: AppState, h: HeaderMap, p: i64, b: Value, kind: &str) -> Reply {
  crate::access::project_action(&s, &h, p, crate::access::ProjectAction::RepositoryAdmin).await?;
  let actor = crate::access::actor(&s, &h).await?;
  let name = required(&b, "name")?;
  let mut scopes = Vec::new();
  for scope in b["scopes"]
    .as_array()
    .ok_or_else(|| AppError::bad_request("token scope is required"))?
  {
    let v = scope.as_str().unwrap_or("").trim().to_lowercase();
    if v.is_empty() {
      continue;
    }
    if ![
      "read_repository",
      "write_repository",
      "read_package",
      "write_package",
      "read_registry",
      "write_registry",
    ]
    .contains(&v.as_str())
      && !(kind == "project_access" && ["read_api", "write_api"].contains(&v.as_str()))
    {
      return Err(AppError::bad_request("token scope is invalid"));
    }
    if !scopes.contains(&v) {
      scopes.push(v);
    }
  }
  if scopes.is_empty() {
    return Err(AppError::bad_request("token scope is required"));
  }
  let expires = if text(&b, "expires_at").is_empty() {
    None
  } else {
    Some(
      chrono::DateTime::parse_from_rfc3339(text(&b, "expires_at"))
        .map_err(|_| AppError::bad_request("invalid expires_at"))?
        .to_utc()
        .naive_utc(),
    )
  };
  let username = if text(&b, "username").trim().is_empty() {
    format!(
      "{}-{}",
      if kind == "deploy" {
        "deploy"
      } else {
        "project"
      },
      name
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase()
    )
  } else {
    text(&b, "username").trim().into()
  };
  let raw = format!(
    "{}{}",
    if kind == "deploy" { "gdt_" } else { "gpat_" },
    crate::auth::secret()
  );
  let row = s
    .repository_access
    .create_token(gity_projects::NewProjectToken {
      project_id: p,
      kind: kind.to_owned(),
      name,
      username,
      token_digest: format!("sha256:{}", crate::auth::token_hash(&raw)),
      scopes: scopes.join(","),
      created_by_user_id: actor.user_id,
      expires_at: expires,
    })
    .await?;
  reply(json!({"project_token":token_view(views::token_json(&row)),"token":raw}))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    post,path="/projects/{id}/access-tokens",tag="credentials",
    params(api::ProjectPath),
    request_body(content=spec::CreateToken),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::CreatedToken>),api::Errors),
    security(("bearer"=[]))
)]
async fn create_access(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  create(s, h, p, b, "project_access").await
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    post,path="/projects/{id}/deploy-tokens",tag="credentials",
    params(api::ProjectPath),
    request_body(content=spec::CreateToken),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::CreatedToken>),api::Errors),
    security(("bearer"=[]))
)]
async fn create_deploy(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  create(s, h, p, b, "deploy").await
}
async fn revoke(s: AppState, h: HeaderMap, p: i64, id: i64, kind: &str) -> Reply {
  crate::access::project_action(&s, &h, p, crate::access::ProjectAction::RepositoryAdmin).await?;
  let row = s
    .repository_access
    .scoped_token(p, id)
    .await?
    .ok_or_else(|| AppError::not_found("resource not found"))?;
  if row.kind != kind {
    return Err(AppError::not_found("project token not found"));
  }
  s.repository_access.revoke_token(row).await?;
  reply(json!({"status":"revoked"}))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    delete,path="/projects/{id}/access-tokens/{token}",tag="credentials",
    params(api::TokenPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<api::Status>),api::Errors),
    security(("bearer"=[]))
)]
async fn revoke_access(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, id)): Path<(i64, i64)>,
) -> Reply {
  revoke(s, h, p, id, "project_access").await
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    delete,path="/projects/{id}/deploy-tokens/{token}",tag="credentials",
    params(api::TokenPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<api::Status>),api::Errors),
    security(("bearer"=[]))
)]
async fn revoke_deploy(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, id)): Path<(i64, i64)>,
) -> Reply {
  revoke(s, h, p, id, "deploy").await
}
fn key_view(mut row: Value) -> Value {
  row["can_push"] = json!(num(&row, "can_push") != 0);
  row
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    get,path="/projects/{id}/deploy-keys",tag="credentials",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::DeployKey>>),api::Errors),
    security(("bearer"=[]))
)]
async fn keys(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  crate::access::project_action(&s, &h, p, crate::access::ProjectAction::RepositoryAdmin).await?;
  reply(json!(
    s.repository_access
      .list_deploy_keys(p)
      .await?
      .into_iter()
      .map(|row| key_view(views::deploy_key_json(&row)))
      .collect::<Vec<_>>()
  ))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    post,path="/projects/{id}/deploy-keys",tag="credentials",
    params(api::ProjectPath),
    request_body(content=spec::CreateDeployKey),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::DeployKey>),api::Errors),
    security(("bearer"=[]))
)]
async fn create_key(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  crate::access::project_action(&s, &h, p, crate::access::ProjectAction::RepositoryAdmin).await?;
  let actor = crate::access::actor(&s, &h).await?;
  let title = required(&b, "title")?;
  let key = required(&b, "public_key")?
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ");
  let (key, fingerprint) = crate::ssh::canonical_key(&key)?;
  let row = s
    .repository_access
    .create_deploy_key(
      p,
      title,
      key,
      fingerprint,
      b["can_push"] == true,
      actor.user_id,
    )
    .await?;
  reply(key_view(views::deploy_key_json(&row)))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    delete,path="/projects/{id}/deploy-keys/{key}",tag="credentials",
    params(api::KeyPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<api::Status>),api::Errors),
    security(("bearer"=[]))
)]
async fn delete_key(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, id)): Path<(i64, i64)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, crate::access::ProjectAction::RepositoryAdmin).await?;
  let key = s
    .repository_access
    .scoped_deploy_key(p, id)
    .await?
    .ok_or_else(|| AppError::not_found("resource not found"))?;
  s.repository_access.delete_deploy_key(key).await?;
  reply(json!({"status":"deleted"}))
}
