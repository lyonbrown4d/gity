use crate::http::app_state::AppState;
use axum::Json;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::{StatusCode, header};
use serde::Serialize;
use std::future::Future;

#[derive(Debug, Clone)]
pub struct CurrentUser {
  pub user_id: String,
  pub organization_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
  pub message: String,
}

impl<S> FromRequestParts<S> for CurrentUser
where
  S: Send + Sync,
  AppState: FromRef<S>,
{
  type Rejection = (StatusCode, Json<AuthErrorResponse>);

  fn from_request_parts(
    parts: &mut axum::http::request::Parts,
    state: &S,
  ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
    let app_state = AppState::from_ref(state);
    let auth_header = parts
      .headers
      .get(header::AUTHORIZATION)
      .and_then(|value| value.to_str().ok())
      .map(|value| value.to_string());
    async move {
      let auth_header = auth_header.ok_or_else(|| unauthorized("missing authorization header"))?;
      let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| unauthorized("invalid authorization scheme"))?
        .to_string();

      let claims = app_state
        .services
        .auth
        .verify_access_token_for_request(token.as_str())
        .await
        .map_err(|_| unauthorized("invalid, expired, or revoked token"))?;

      Ok(Self {
        user_id: claims.sub,
        organization_id: claims.org,
      })
    }
  }
}

fn unauthorized(message: &str) -> (StatusCode, Json<AuthErrorResponse>) {
  (
    StatusCode::UNAUTHORIZED,
    Json(AuthErrorResponse {
      message: message.to_string(),
    }),
  )
}
