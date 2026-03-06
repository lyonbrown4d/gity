use crate::http::app_state::AppState;
use crate::http::auth::ErrorResponse;
use crate::http::pagination::{resolve_pagination, to_page, to_single_page};
use crate::security::current_user::CurrentUser;
use crate::service::user_service::{
  CreateUserInput, UpdateCurrentUserInput, UpdateUserForAdminInput, UserServiceError,
};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use domain::api::crud::parse_csv_ids;
use domain::api::response::{ApiResponse, EmptyData};
use domain::api::user::{
  CreateUserRequest, ListUsersQuery, UpdateMeRequest, UpdateUserRequest, UserView,
};
use domain::page::Page;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
  get,
  path = "/",
  tag = "Users",
  params(ListUsersQuery),
  responses(
    (status = 200, description = "List active users (super admin only)", body = ApiResponse<Page<UserView>>),
    (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn list_users(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Query(query): Query<ListUsersQuery>,
) -> Result<(StatusCode, Json<Page<UserView>>), (StatusCode, Json<ErrorResponse>)> {
  let ids = parse_csv_ids(query.ids.as_deref());
  if ids.is_empty() {
    let pagination = resolve_pagination(query.page, query.page_size.or(query.limit), 100, 200)?;
    let (users, total) = state
      .services
      .user
      .list_users_page_for_admin(
        current_user.user_id.as_str(),
        pagination.page,
        pagination.page_size,
      )
      .await
      .map_err(map_user_service_error)?;

    let data = users
      .iter()
      .map(|model| user_view(model, state.services.user.is_super_admin(model)))
      .collect();
    return Ok((StatusCode::OK, Json(to_page(data, total, pagination))));
  }

  let users = state
    .services
    .user
    .list_users_by_ids_for_admin(current_user.user_id.as_str(), ids)
    .await
    .map_err(map_user_service_error)?;

  let data: Vec<UserView> = users
    .iter()
    .map(|model| user_view(model, state.services.user.is_super_admin(model)))
    .collect();
  Ok((StatusCode::OK, Json(to_single_page(data))))
}

#[utoipa::path(
  get,
  path = "/me",
  tag = "Users",
  responses(
    (status = 200, description = "Current user", body = ApiResponse<UserView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Users",
  responses(
    (status = 200, description = "Get user by id (super admin) or me", body = ApiResponse<UserView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "User not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Users",
  request_body = UpdateMeRequest,
  responses(
    (status = 200, description = "Current user updated", body = ApiResponse<UserView>),
    (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Conflict", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  patch,
  path = "/{user_id}",
  tag = "Users",
  request_body = UpdateUserRequest,
  responses(
    (status = 200, description = "User updated (super admin)", body = ApiResponse<UserView>),
    (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "User not found", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Conflict", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn update_user(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(user_id): Path<String>,
  Json(payload): Json<UpdateUserRequest>,
) -> Result<(StatusCode, Json<UserView>), (StatusCode, Json<ErrorResponse>)> {
  if user_id == "me" {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "use /users/me to update current user".to_string(),
      }),
    ));
  }

  let user = state
    .services
    .user
    .update_user_for_admin(
      current_user.user_id.as_str(),
      UpdateUserForAdminInput {
        user_id,
        username: payload.username,
        email: payload.email,
        password: payload.password,
        status: parse_user_status(payload.status.as_deref())?,
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
  tag = "Users",
  request_body = CreateUserRequest,
  responses(
    (status = 201, description = "User created (super admin only)", body = ApiResponse<UserView>),
    (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Conflict", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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

#[utoipa::path(
  delete,
  path = "/{user_id}",
  tag = "Users",
  responses(
    (status = 200, description = "User deleted (super admin)", body = ApiResponse<EmptyData>),
    (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "User not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn delete_user(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(user_id): Path<String>,
) -> Result<(StatusCode, Json<EmptyData>), (StatusCode, Json<ErrorResponse>)> {
  if user_id == "me" {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "use your profile to update current user".to_string(),
      }),
    ));
  }

  state
    .services
    .user
    .delete_user_for_admin(current_user.user_id.as_str(), user_id.as_str())
    .await
    .map_err(map_user_service_error)?;

  Ok((StatusCode::OK, Json(EmptyData {})))
}

pub fn user_routes() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes![list_users])
    .routes(routes![create_user])
    .routes(routes![get_me])
    .routes(routes![get_user])
    .routes(routes![update_me])
    .routes(routes![update_user])
    .routes(routes![delete_user])
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

fn parse_user_status(
  status: Option<&str>,
) -> Result<Option<entity::users::UserStatus>, (StatusCode, Json<ErrorResponse>)> {
  let Some(status) = status else {
    return Ok(None);
  };

  let normalized = status.trim().to_ascii_lowercase();
  if normalized.is_empty() {
    return Ok(None);
  }

  match normalized.as_str() {
    "active" => Ok(Some(entity::users::UserStatus::Active)),
    "disabled" => Ok(Some(entity::users::UserStatus::Disabled)),
    _ => Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "status must be one of: active, disabled".to_string(),
      }),
    )),
  }
}
