use crate::http::app_state::AppState;
use crate::security::current_user::CurrentUser;
use crate::service::auth_service::{AuthPayload, AuthServiceError, LoginInput, RegisterInput};
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use domain::api::response::{ApiResponse, EmptyData};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
  pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RegisterRequest {
  pub username: String,
  pub email: String,
  pub password: String,
  pub organization_name: Option<String>,
  pub organization_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
  pub username: String,
  pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
  pub user_id: String,
  pub username: String,
  pub organization_id: Option<String>,
  pub organization_name: Option<String>,
  pub token: String,
  pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefreshRequest {
  pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LogoutRequest {
  pub refresh_token: Option<String>,
}

#[utoipa::path(
  post,
  path = "/register",
  tag = "Auth",
  request_body = RegisterRequest,
  responses(
    (status = 201, description = "User and default organization created", body = ApiResponse<AuthResponse>),
    (status = 409, description = "User or organization already exists", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn register(
  State(state): State<AppState>,
  Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<ErrorResponse>)> {
  let auth = state
    .services
    .auth
    .register(RegisterInput {
      username: payload.username,
      email: payload.email,
      password: payload.password,
      organization_name: payload.organization_name,
      organization_key: payload.organization_key,
    })
    .await
    .map_err(map_auth_error)?;

  Ok((StatusCode::CREATED, Json(auth_response(auth))))
}

#[utoipa::path(
  post,
  path = "/login",
  tag = "Auth",
  request_body = LoginRequest,
  responses(
    (status = 200, description = "Login success", body = ApiResponse<AuthResponse>),
    (status = 401, description = "Invalid credentials", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn login(
  State(state): State<AppState>,
  Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<ErrorResponse>)> {
  let auth = state
    .services
    .auth
    .login(LoginInput {
      username: payload.username,
      password: payload.password,
    })
    .await
    .map_err(map_auth_error)?;

  Ok((StatusCode::OK, Json(auth_response(auth))))
}

#[utoipa::path(
  post,
  path = "/refresh",
  tag = "Auth",
  request_body = RefreshRequest,
  responses(
    (status = 200, description = "Token refreshed", body = ApiResponse<AuthResponse>),
    (status = 401, description = "Invalid refresh token", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn refresh(
  State(state): State<AppState>,
  Json(payload): Json<RefreshRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<ErrorResponse>)> {
  let auth = state
    .services
    .auth
    .refresh(payload.refresh_token.as_str())
    .await
    .map_err(map_auth_error)?;

  Ok((StatusCode::OK, Json(auth_response(auth))))
}

#[utoipa::path(
  post,
  path = "/logout",
  tag = "Auth",
  request_body = LogoutRequest,
  responses(
    (status = 200, description = "Logged out", body = ApiResponse<EmptyData>),
    (status = 401, description = "Invalid token", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn logout(
  State(state): State<AppState>,
  current_user: CurrentUser,
  headers: HeaderMap,
  payload: Option<Json<LogoutRequest>>,
) -> Result<(StatusCode, Json<EmptyData>), (StatusCode, Json<ErrorResponse>)> {
  let access_token = headers
    .get(header::AUTHORIZATION)
    .and_then(|value| value.to_str().ok())
    .and_then(|value| value.strip_prefix("Bearer "))
    .ok_or_else(|| {
      (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
          message: "missing authorization header".to_string(),
        }),
      )
    })?;

  let refresh_token = payload.and_then(|Json(payload)| payload.refresh_token);

  state
    .services
    .auth
    .logout(
      current_user.user_id.as_str(),
      access_token,
      refresh_token.as_deref(),
    )
    .await
    .map_err(map_auth_error)?;

  Ok((StatusCode::OK, Json(EmptyData {})))
}

pub fn auth_routes() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes![register])
    .routes(routes![login])
    .routes(routes![refresh])
    .routes(routes![logout])
}

fn auth_response(payload: AuthPayload) -> AuthResponse {
  AuthResponse {
    user_id: payload.user_id,
    username: payload.username,
    organization_id: payload.organization_id,
    organization_name: payload.organization_name,
    token: payload.token,
    refresh_token: payload.refresh_token,
  }
}

fn map_auth_error(err: AuthServiceError) -> (StatusCode, Json<ErrorResponse>) {
  let (status, message) = match err {
    AuthServiceError::Conflict(message) => (StatusCode::CONFLICT, message),
    AuthServiceError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
    AuthServiceError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
  };

  (status, Json(ErrorResponse { message }))
}
