use crate::http::app_state::AppState;
use crate::http::auth::ErrorResponse;
use crate::security::current_user::CurrentUser;
use crate::service::user_service::{CreateUserInput, UpdateCurrentUserInput, UserServiceError};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use domain::api::crud::parse_csv_ids;
use domain::api::user::{CreateUserRequest, ListUsersQuery, UpdateMeRequest, UserView};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
  get,
  path = "/",
  params(ListUsersQuery),
  responses(
    (status = 200, description = "List active users (super admin only)", body = [UserView]),
    (status = 400, description = "Bad request", body = ErrorResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn list_users(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Query(query): Query<ListUsersQuery>,
) -> Result<(StatusCode, Json<Vec<UserView>>), (StatusCode, Json<ErrorResponse>)> {
  let ids = parse_csv_ids(query.ids.as_deref());
  let users = if ids.is_empty() {
    state
      .services
      .user
      .list_users_for_admin(current_user.user_id.as_str(), query.limit)
      .await
      .map_err(map_user_service_error)?
  } else {
    state
      .services
      .user
      .list_users_by_ids_for_admin(current_user.user_id.as_str(), ids)
      .await
      .map_err(map_user_service_error)?
  };

  let data = users
    .iter()
    .map(|model| user_view(model, state.services.user.is_super_admin(model)))
    .collect();

  Ok((StatusCode::OK, Json(data)))
}

#[utoipa::path(
  get,
  path = "/me",
  responses(
    (status = 200, description = "Current user", body = UserView),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn get_me(
  State(state): State<AppState>,
  current_user: CurrentUser,
) -> Result<(StatusCode, Json<UserView>), (StatusCode, Json<ErrorResponse>)> {
  let user = state
    .services
    .user
    .get_current_user(current_user.user_id.as_str())
    .await
    .map_err(map_user_service_error)?;

  let is_super_admin = state.services.user.is_super_admin(&user);
  Ok((StatusCode::OK, Json(user_view(&user, is_super_admin))))
}

#[utoipa::path(
  get,
  path = "/{user_id}",
  responses(
    (status = 200, description = "Get user by id (super admin) or me", body = UserView),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "User not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn get_user(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(user_id): Path<String>,
) -> Result<(StatusCode, Json<UserView>), (StatusCode, Json<ErrorResponse>)> {
  let user = if user_id == "me" {
    state
      .services
      .user
      .get_current_user(current_user.user_id.as_str())
      .await
      .map_err(map_user_service_error)?
  } else {
    state
      .services
      .user
      .get_user_by_id_for_admin(current_user.user_id.as_str(), user_id.as_str())
      .await
      .map_err(map_user_service_error)?
  };

  let is_super_admin = state.services.user.is_super_admin(&user);
  Ok((StatusCode::OK, Json(user_view(&user, is_super_admin))))
}

#[utoipa::path(
  patch,
  path = "/me",
  request_body = UpdateMeRequest,
  responses(
    (status = 200, description = "Current user updated", body = UserView),
    (status = 400, description = "Bad request", body = ErrorResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 409, description = "Conflict", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn update_me(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Json(payload): Json<UpdateMeRequest>,
) -> Result<(StatusCode, Json<UserView>), (StatusCode, Json<ErrorResponse>)> {
  let user = state
    .services
    .user
    .update_current_user(
      current_user.user_id.as_str(),
      UpdateCurrentUserInput {
        username: payload.username,
        email: payload.email,
        password: payload.password,
      },
    )
    .await
    .map_err(map_user_service_error)?;

  let is_super_admin = state.services.user.is_super_admin(&user);
  Ok((StatusCode::OK, Json(user_view(&user, is_super_admin))))
}

#[utoipa::path(
  post,
  path = "/",
  request_body = CreateUserRequest,
  responses(
    (status = 201, description = "User created (super admin only)", body = UserView),
    (status = 400, description = "Bad request", body = ErrorResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 409, description = "Conflict", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn create_user(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserView>), (StatusCode, Json<ErrorResponse>)> {
  let created = state
    .services
    .user
    .create_user_for_admin(
      current_user.user_id.as_str(),
      CreateUserInput {
        username: payload.username,
        email: payload.email,
        password: payload.password,
      },
    )
    .await
    .map_err(map_user_service_error)?;

  Ok((
    StatusCode::CREATED,
    Json(user_view(
      &created,
      state.services.user.is_super_admin(&created),
    )),
  ))
}

pub fn user_routes() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes![list_users])
    .routes(routes![create_user])
    .routes(routes![get_me])
    .routes(routes![get_user])
    .routes(routes![update_me])
}

fn user_view(model: &entity::users::Model, is_super_admin: bool) -> UserView {
  UserView {
    id: model.id.clone(),
    username: model.username.clone(),
    email: model.email.clone(),
    status: match model.status.clone() {
      entity::users::UserStatus::Active => "active".to_string(),
      entity::users::UserStatus::Disabled => "disabled".to_string(),
    },
    is_super_admin,
  }
}

fn map_user_service_error(err: UserServiceError) -> (StatusCode, Json<ErrorResponse>) {
  let (status, message) = match err {
    UserServiceError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
    UserServiceError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
    UserServiceError::Forbidden(message) => (StatusCode::FORBIDDEN, message),
    UserServiceError::NotFound(message) => (StatusCode::NOT_FOUND, message),
    UserServiceError::Conflict(message) => (StatusCode::CONFLICT, message),
    UserServiceError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
  };

  (status, Json(ErrorResponse { message }))
}
