use crate::http::app_state::AppState;
use crate::security::jwt::verify_access_token;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::{StatusCode, header};
use axum::Json;
use serde::Serialize;
use std::future::{Future, ready};

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
    let secret = app_state
      .config
      .auth
      .as_ref()
      .and_then(|auth| auth.jwt_secret.clone())
      .filter(|secret| !secret.is_empty())
      .unwrap_or_else(|| "gity-dev-secret-change-me".to_string());

    let auth_header = match parts
      .headers
      .get(header::AUTHORIZATION)
      .and_then(|value| value.to_str().ok())
    {
      Some(header) => header,
      None => return ready(Err(unauthorized("missing authorization header"))),
    };

    let token = match auth_header.strip_prefix("Bearer ") {
      Some(token) => token,
      None => return ready(Err(unauthorized("invalid authorization scheme"))),
    };

    let claims = verify_access_token(secret.as_str(), token)
      .map_err(|_| unauthorized("invalid or expired token"));

    let result = claims.map(|claims| Self {
      user_id: claims.sub,
      organization_id: claims.org,
    });

    ready(result)
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
