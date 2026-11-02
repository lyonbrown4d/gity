use argon2::{
  Argon2,
  password_hash::{PasswordVerifier, phc::PasswordHash},
};
use axum::{
  Json,
  extract::{Path, State},
  http::HeaderMap,
};
use chrono::Utc;
use gity_core::{AppError, ok};
use gity_identity::PendingAuthorization;
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;

use crate::{AppState, auth, openapi::schemas as api};

#[derive(serde::Serialize, ToSchema)]
struct PendingResponse {
  request_id: String,
  client_name: String,
  scopes: Vec<String>,
  resource: String,
  requests_admin_mode: bool,
}

impl From<PendingAuthorization> for PendingResponse {
  fn from(value: PendingAuthorization) -> Self {
    Self {
      request_id: value.request_id,
      client_name: value.client_name,
      scopes: value.scopes,
      resource: value.resource,
      requests_admin_mode: value.requests_admin_mode,
    }
  }
}

#[utoipa::path(
  get,
  path = "/api/v1/oauth/requests/{request_id}",
  tag = "oauth",
  description = "Returns the pending consent details and binds the request to the current browser session.",
  params(("request_id" = String, Path, description = "UUID v7 consent request identifier")),
  responses((status = 200, description = "Consent details", body = api::Envelope<PendingResponse>), api::Errors),
  security(("bearer" = []))
)]
pub(super) async fn pending(
  State(state): State<AppState>,
  Path(request_id): Path<String>,
  headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
  let principal = session_principal(&state, &headers).await?;
  let details = state
    .oauth
    .pending_details(&request_id, principal.user_id)
    .await
    .map_err(super::map_oauth)?;
  let response = PendingResponse::from(details);
  Ok(ok(serde_json::to_value(response).map_err(|_| {
    AppError::internal("OAuth response encoding failed")
  })?))
}

#[derive(Deserialize, ToSchema)]
pub(super) struct DecisionRequest {
  approve: bool,
  #[serde(default)]
  #[schema(format = Password, max_length = 1024, write_only = true)]
  password: Option<String>,
}

#[derive(serde::Serialize, ToSchema)]
struct DecisionResponse {
  redirect_uri: String,
}

#[utoipa::path(
  post,
  path = "/api/v1/oauth/requests/{request_id}",
  tag = "oauth",
  description = "Approves or denies consent. Admin mode requires the current administrator password.",
  params(("request_id" = String, Path, description = "UUID v7 consent request identifier")),
  request_body = DecisionRequest,
  responses((status = 200, description = "Callback redirect URI", body = api::Envelope<DecisionResponse>), api::Errors),
  security(("bearer" = []))
)]
pub(super) async fn decide(
  State(state): State<AppState>,
  Path(request_id): Path<String>,
  headers: HeaderMap,
  Json(input): Json<DecisionRequest>,
) -> Result<Json<Value>, AppError> {
  let principal = session_principal(&state, &headers).await?;
  let details = state
    .oauth
    .pending_details(&request_id, principal.user_id)
    .await
    .map_err(super::map_oauth)?;
  let admin_reauthenticated = if input.approve && details.requests_admin_mode {
    let password = input
      .password
      .filter(|value| !value.is_empty() && value.len() <= 1024)
      .ok_or_else(|| AppError::unauthorized("administrator reauthentication required"))?;
    verify_password(
      &state,
      principal.user_id,
      principal.is_super_admin,
      password,
    )
    .await?
  } else {
    false
  };
  let redirect_uri = state
    .oauth
    .decide(
      &request_id,
      principal.user_id,
      input.approve,
      admin_reauthenticated,
    )
    .await
    .map_err(super::map_oauth)?;
  Ok(ok(
    serde_json::to_value(DecisionResponse { redirect_uri })
      .map_err(|_| AppError::internal("OAuth response encoding failed"))?,
  ))
}

async fn session_principal(
  state: &AppState,
  headers: &HeaderMap,
) -> Result<gity_core::Principal, AppError> {
  let token =
    auth::bearer(headers).ok_or_else(|| AppError::unauthorized("authentication required"))?;
  if token.starts_with("gity_oauth_") {
    return Err(AppError::unauthorized(
      "OAuth consent requires a browser session",
    ));
  }
  let user = state
    .identity
    .user_for_access_session(&auth::token_hash(token), Utc::now().naive_utc())
    .await?
    .ok_or_else(|| AppError::unauthorized("valid browser session required"))?;
  if !user.is_active() {
    return Err(AppError::unauthorized("valid browser session required"));
  }
  Ok(gity_core::Principal {
    user_id: user.id,
    username: user.username,
    is_super_admin: user.is_super_admin,
  })
}

async fn verify_password(
  state: &AppState,
  user_id: i64,
  is_admin: bool,
  password: String,
) -> Result<bool, AppError> {
  if !is_admin {
    return Err(AppError::forbidden("administrator access required"));
  }
  let encoded = state
    .identity
    .password_hash(user_id)
    .await?
    .ok_or_else(|| AppError::unauthorized("invalid password"))?;
  let verified = tokio::task::spawn_blocking(move || {
    PasswordHash::new(&encoded).is_ok_and(|hash| {
      Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .is_ok()
    })
  })
  .await
  .map_err(|_| AppError::internal("password verification failed"))?;
  if !verified {
    return Err(AppError::unauthorized("invalid password"));
  }
  Ok(true)
}
