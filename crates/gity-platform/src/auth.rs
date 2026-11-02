use crate::AppState;
pub(crate) mod bootstrap;
mod challenge;
mod sessions;
use crate::openapi::{schemas as api, user_schemas as spec};
use argon2::{
  Argon2,
  password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use axum::{Json, Router, extract::State, http::HeaderMap};
pub(crate) use challenge::basic_challenge;
use gity_core::{AppError, Principal, new_id, ok};
use serde::Deserialize;
use serde_json::{Value, json};
use sessions::{create_account, session};
use sha2::{Digest, Sha256};

pub fn routes() -> Router<AppState> {
  documented_routes().split_for_parts().0
}
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(login))
    .routes(utoipa_axum::routes!(register))
    .routes(utoipa_axum::routes!(refresh))
    .routes(utoipa_axum::routes!(logout))
}
pub fn token_hash(token: &str) -> String {
  hex::encode(Sha256::digest(token.as_bytes()))
}
pub fn secret() -> String {
  format!(
    "{}{}",
    uuid::Uuid::new_v4().simple(),
    uuid::Uuid::new_v4().simple()
  )
}
pub fn bearer(headers: &HeaderMap) -> Option<&str> {
  headers
    .get("authorization")?
    .to_str()
    .ok()?
    .strip_prefix("Bearer ")
}
pub async fn principal(state: &AppState, headers: &HeaderMap) -> Result<Principal, AppError> {
  let token = bearer(headers).ok_or_else(|| AppError::unauthorized("authentication required"))?;
  principal_token(state, token).await
}
pub async fn principal_token(state: &AppState, token: &str) -> Result<Principal, AppError> {
  if token.starts_with("gity_oauth_") {
    let audience = format!("{}/mcp", state.settings.base_url.trim_end_matches('/'));
    let identity = state
      .oauth
      .validate_bearer(token, &audience)
      .await
      .map_err(|_| AppError::unauthorized("invalid or expired credentials"))?
      .ok_or_else(|| AppError::unauthorized("invalid or expired credentials"))?;
    return Ok(Principal {
      user_id: identity.user_id,
      username: identity.username,
      is_super_admin: identity.is_super_admin,
    });
  }
  let repository = &state.identity;
  if let Some(user) = repository
    .user_for_access_session(&token_hash(token), chrono::Utc::now().naive_utc())
    .await?
  {
    return from_identity(user);
  }
  // Personal tokens are stored only as digests. Refresh tokens never authenticate requests.
  if let Some(user) = repository
    .user_for_personal_token(&format!("sha256:{}", token_hash(token)))
    .await?
  {
    return from_identity(user);
  }
  Err(AppError::unauthorized("invalid or expired credentials"))
}
fn from_identity(user: gity_identity::UserIdentity) -> Result<Principal, AppError> {
  if !user.is_active() {
    return Err(AppError::unauthorized("invalid or expired credentials"));
  }
  Ok(Principal {
    user_id: user.id,
    username: user.username,
    is_super_admin: user.is_super_admin,
  })
}
fn from_user(row: &Value) -> Result<Principal, AppError> {
  if row["status"].as_str() != Some("active") {
    return Err(AppError::unauthorized("invalid or expired credentials"));
  }
  Ok(Principal {
    user_id: row["id"]
      .as_i64()
      .ok_or_else(|| AppError::internal("invalid user identifier"))?,
    username: row["username"].as_str().unwrap_or_default().into(),
    is_super_admin: row["is_super_admin"]
      .as_bool()
      .unwrap_or_else(|| row["is_super_admin"].as_i64() == Some(1)),
  })
}
#[derive(Deserialize, utoipa::ToSchema)]
struct Credentials {
  username: String,
  #[schema(format=Password,write_only,min_length=8,max_length=1024)]
  password: String,
}
#[derive(Deserialize, utoipa::ToSchema)]
struct Refresh {
  refresh_token: String,
}
#[utoipa::path(
    post,path="/auth/login",tag="auth",
    description="Authenticates an existing password-backed account.",
    request_body=Credentials,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Session>),api::Errors),
    security((), ("bearer"=[]))
)]
async fn login(
  State(state): State<AppState>,
  Json(body): Json<Credentials>,
) -> Result<Json<Value>, AppError> {
  validate_credentials(&body)?;
  let repository = &state.identity;
  let user = repository
    .user_model_by_username(body.username.trim())
    .await?;
  let Some(user) = user else {
    return Err(AppError::unauthorized("invalid username or password"));
  };
  let encoded = repository.password_hash(user.id).await?.ok_or_else(|| {
    AppError::unauthorized("password not configured; administrator must run set-password")
  })?;
  let verified = tokio::task::spawn_blocking(move || {
    PasswordHash::new(&encoded).is_ok_and(|hash| {
      Argon2::default()
        .verify_password(body.password.as_bytes(), &hash)
        .is_ok()
    })
  })
  .await
  .map_err(|_| AppError::internal("password verification failed"))?;
  if !verified {
    return Err(AppError::unauthorized("invalid username or password"));
  }
  session(
    &state,
    &json!({
        "id": user.id,
        "username": user.username,
        "status": user.status,
        "is_super_admin": user.is_super_admin,
    }),
    None,
  )
  .await
}
#[utoipa::path(
    post,path="/auth/register",tag="auth",
    description="Creates a password-backed account and personal organization.",
    request_body=Credentials,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Session>),api::Errors),
    security((), ("bearer"=[]))
)]
async fn register(
  State(state): State<AppState>,
  Json(body): Json<Credentials>,
) -> Result<Json<Value>, AppError> {
  validate_credentials(&body)?;
  create_account(&state, body).await
}
pub(crate) fn validate_username(raw: &str) -> Result<&str, AppError> {
  let username = raw.trim();
  if username.is_empty()
    || username.len() > 64
    || !username
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
  {
    return Err(AppError::bad_request(
      "username must use 1–64 letters, digits, dots, dashes or underscores",
    ));
  }
  Ok(username)
}
fn validate_credentials(body: &Credentials) -> Result<(), AppError> {
  validate_username(&body.username)?;
  if body.password.len() < 8 || body.password.len() > 1024 {
    return Err(AppError::bad_request("password must contain 8–1024 bytes"));
  }
  Ok(())
}
pub async fn hash_password(password: String) -> Result<String, AppError> {
  tokio::task::spawn_blocking(move || {
    Argon2::default()
      .hash_password(password.as_bytes())
      .map(|v| v.to_string())
      .map_err(|_| AppError::internal("password hashing failed"))
  })
  .await
  .map_err(|_| AppError::internal("password hashing failed"))?
}
#[utoipa::path(
    post,path="/auth/refresh",tag="auth",
    description="Rotates a one-use refresh token; replay fails with 401.",
    request_body=Refresh,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Session>),api::Errors),
    security((), ("bearer"=[]))
)]
async fn refresh(
  State(state): State<AppState>,
  Json(body): Json<Refresh>,
) -> Result<Json<Value>, AppError> {
  let hash = token_hash(&body.refresh_token);
  let Some((user, family)) = state
    .identity
    .consume_refresh_token(&hash, chrono::Utc::now().naive_utc())
    .await?
  else {
    return Err(AppError::unauthorized("invalid refresh token"));
  };
  session(
    &state,
    &json!({
        "id": user.id,
        "username": user.username,
        "status": user.status,
        "is_super_admin": user.is_super_admin,
    }),
    Some(family),
  )
  .await
}
#[utoipa::path(
    post,path="/auth/logout",tag="auth",
    description="Revokes the session family referenced by bearer or refresh credentials.",
    request_body=spec::Logout,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::OkResult>),api::Errors),
    security((), ("bearer"=[]))
)]
async fn logout(
  State(state): State<AppState>,
  headers: HeaderMap,
  Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
  for token in [bearer(&headers), body["refresh_token"].as_str()]
    .into_iter()
    .flatten()
  {
    let repository = &state.identity;
    if let Some(family) = repository.session_family(&token_hash(token)).await? {
      repository.revoke_session_family(&family).await?;
    }
  }
  Ok(ok(json!({"ok":true})))
}
