use axum::{
  Form, Json,
  extract::{State, rejection::FormRejection},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use gity_identity::{OAuthError, TokenReply, TokenRequest};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::AppState;

#[derive(Deserialize, ToSchema)]
pub(super) struct TokenRequestForm {
  grant_type: String,
  #[schema(max_length = 128)]
  code: Option<String>,
  #[schema(max_length = 512, write_only = true)]
  refresh_token: Option<String>,
  #[schema(max_length = 128)]
  client_id: String,
  #[schema(max_length = 2048)]
  redirect_uri: Option<String>,
  #[schema(min_length = 43, max_length = 128, write_only = true)]
  code_verifier: Option<String>,
  #[schema(max_length = 2048)]
  resource: Option<String>,
}

impl From<TokenRequestForm> for TokenRequest {
  fn from(form: TokenRequestForm) -> Self {
    Self {
      grant_type: form.grant_type,
      code: form.code,
      refresh_token: form.refresh_token,
      client_id: form.client_id,
      redirect_uri: form.redirect_uri,
      code_verifier: form.code_verifier,
      resource: form.resource,
    }
  }
}

#[derive(serde::Serialize, ToSchema)]
struct TokenResponse {
  access_token: String,
  token_type: String,
  expires_in: u64,
  refresh_token: String,
  scope: String,
}

impl From<TokenReply> for TokenResponse {
  fn from(reply: TokenReply) -> Self {
    Self {
      access_token: reply.access_token,
      token_type: reply.token_type.into(),
      expires_in: reply.expires_in,
      refresh_token: reply.refresh_token,
      scope: reply.scope,
    }
  }
}

#[derive(serde::Serialize, ToSchema)]
struct OAuthErrorResponse {
  error: String,
}

#[utoipa::path(
  post,
  path = "/oauth/token",
  tag = "oauth",
  description = "Exchanges a PKCE authorization code or rotates a one-use refresh token.",
  request_body(content = TokenRequestForm, content_type = "application/x-www-form-urlencoded"),
  responses(
    (status = 200, description = "OAuth bearer and refresh tokens", body = TokenResponse),
    (status = 400, description = "OAuth request or grant is invalid", body = OAuthErrorResponse),
    (status = 500, description = "OAuth service unavailable", body = OAuthErrorResponse)
  ),
  security(())
)]
pub(super) async fn token(
  State(state): State<AppState>,
  input: Result<Form<TokenRequestForm>, FormRejection>,
) -> Response {
  let Form(input) = match input {
    Ok(form) => form,
    Err(_) => return no_store(oauth_error(StatusCode::BAD_REQUEST, "invalid_request")),
  };
  if !matches!(
    input.grant_type.as_str(),
    "authorization_code" | "refresh_token"
  ) {
    return no_store(oauth_error(
      StatusCode::BAD_REQUEST,
      "unsupported_grant_type",
    ));
  }
  if input.client_id.is_empty()
    || match input.grant_type.as_str() {
      "authorization_code" => {
        input.code.is_none() || input.redirect_uri.is_none() || input.code_verifier.is_none()
      }
      _ => input.refresh_token.is_none(),
    }
  {
    return no_store(oauth_error(StatusCode::BAD_REQUEST, "invalid_request"));
  }
  let response = match state.oauth.token(input.into()).await {
    Ok(reply) => serde_json::to_value(TokenResponse::from(reply))
      .map(|body| (StatusCode::OK, Json(body)).into_response())
      .unwrap_or_else(|_| oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error")),
    Err(OAuthError::Invalid) => oauth_error(StatusCode::BAD_REQUEST, "invalid_grant"),
    Err(OAuthError::Expired) => oauth_error(StatusCode::BAD_REQUEST, "invalid_grant"),
    Err(OAuthError::Database(_)) | Err(OAuthError::Configuration) => {
      oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error")
    }
  };
  no_store(response)
}

fn no_store(mut response: Response) -> Response {
  response
    .headers_mut()
    .insert("cache-control", "no-store".parse().unwrap());
  response
    .headers_mut()
    .insert("pragma", "no-cache".parse().unwrap());
  response
}

fn oauth_error(status: StatusCode, code: &str) -> Response {
  (
    status,
    Json(OAuthErrorResponse {
      error: code.to_owned(),
    }),
  )
    .into_response()
}

#[derive(Deserialize, ToSchema)]
pub(super) struct RevokeRequest {
  #[schema(min_length = 1, max_length = 512, write_only = true)]
  token: String,
}

#[utoipa::path(
  post,
  path = "/oauth/revoke",
  tag = "oauth",
  description = "Revokes an access or refresh token. Unknown tokens are ignored as required by RFC 7009.",
  request_body(content = RevokeRequest, content_type = "application/x-www-form-urlencoded"),
  responses((status = 200, description = "Token revoked or already unknown"), (status = 400, description = "Invalid revocation request", body = OAuthErrorResponse), (status = 500, description = "OAuth service unavailable", body = OAuthErrorResponse)),
  security(())
)]
pub(super) async fn revoke(
  State(state): State<AppState>,
  input: Result<Form<RevokeRequest>, FormRejection>,
) -> Response {
  let Ok(Form(input)) = input else {
    return oauth_error(StatusCode::BAD_REQUEST, "invalid_request");
  };
  if input.token.is_empty() || input.token.len() > 512 {
    return oauth_error(StatusCode::BAD_REQUEST, "invalid_request");
  }
  match state.oauth.revoke(&input.token).await {
    Ok(()) => StatusCode::OK.into_response(),
    Err(_) => oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error"),
  }
}
