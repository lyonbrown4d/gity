//! Personal access token endpoints for the user resource.
use super::users::timestamp;
use crate::openapi::{schemas as api, user_schemas as spec};
use crate::{AppState, auth};
use axum::{
  Json,
  extract::{Path, State},
  http::HeaderMap,
};
use gity_core::{AppError, ok};
use serde_json::{Value, json};
type Reply = Result<Json<Value>, AppError>;

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!(
    tokens,
    create_token,
    revoke_token
  ))
}

async fn own_tokens(s: &AppState, h: &HeaderMap, id: i64) -> Result<(), AppError> {
  let p = auth::principal(s, h).await?;
  if p.user_id != id && !p.is_super_admin {
    return Err(AppError::forbidden(
      "cannot access another user's credentials",
    ));
  }
  Ok(())
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    get,path="/users/{id}/tokens",tag="users",
    description="Only this user or an administrator can list token metadata. Raw tokens and hashes are omitted.",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::UserToken>>),api::Errors),
    security(("bearer"=[]))
)]
async fn tokens(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i64>) -> Reply {
  own_tokens(&s, &h, id).await?;
  let tokens = s
    .identity
    .personal_tokens(id)
    .await?
    .into_iter()
    .map(|token| {
      json!({
          "id": token.id,
          "user_id": token.user_id,
          "name": token.name,
          "created_at": timestamp(token.created_at),
          "updated_at": timestamp(token.updated_at),
      })
    })
    .collect::<Vec<_>>();
  Ok(ok(tokens))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    post,path="/users/{id}/tokens",tag="users",
    description="Only this user or an administrator; raw token returned once and stored hashed.",
    params(api::ProjectPath),
    request_body=spec::CreateUserToken,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::CreatedUserToken>),api::Errors),
    security(("bearer"=[]))
)]
async fn create_token(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  own_tokens(&s, &h, id).await?;
  let name = b["name"]
    .as_str()
    .filter(|v| !v.is_empty() && *v != "access" && *v != "refresh")
    .ok_or_else(|| AppError::bad_request("personal token name required"))?;
  let token = auth::secret();
  let created = s
    .identity
    .create_personal_token(
      gity_core::new_id(),
      id,
      name.to_owned(),
      format!("sha256:{}", auth::token_hash(&token)),
    )
    .await?;
  let row = json!({
      "id": created.id,
      "user_id": created.user_id,
      "name": created.name,
      "created_at": timestamp(created.created_at),
      "updated_at": timestamp(created.updated_at),
      "token": token,
  });
  Ok(ok(row))
}

#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    delete,path="/users/{id}/tokens/{token}",tag="users",
    description="Only this user or an administrator can revoke a personal token. The token stops authenticating immediately.",
    params(("id"=i64, Path, description="User identifier"), ("token"=i64, Path, description="Personal token identifier")),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::OkResult>),api::Errors),
    security(("bearer"=[]))
)]
async fn revoke_token(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((id, token)): Path<(i64, i64)>,
) -> Reply {
  own_tokens(&s, &h, id).await?;
  if !s.identity.delete_personal_token(id, token).await? {
    return Err(AppError::not_found("personal token not found"));
  }
  Ok(ok(json!({ "ok": true })))
}
