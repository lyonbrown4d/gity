use crate::app_state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserCreate {
  pub username: String,
  pub email: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserUpdate {
  pub username: Option<String>,
  pub email: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserView {
  pub id: u64,
  pub username: String,
  pub email: String,
}

#[utoipa::path(
    post,
    path = "/user",
    request_body = UserCreate,
    responses(
        (status = 201, description = "User created", body = UserView)
    )
)]
pub async fn create_user(
  State(_state): State<AppState>,
  Json(payload): Json<UserCreate>,
) -> (StatusCode, Json<UserView>) {
  let user = UserView {
    id: 1,
    username: payload.username,
    email: payload.email,
  };
  (StatusCode::CREATED, Json(user))
}

#[utoipa::path(
  get,
  path = "/{id}",
  responses(
        (status = 200, description = "Get user", body = UserView),
        (status = 404, description = "User not found")
  )
)]
pub async fn get_user(
  State(_state): State<AppState>,
  axum::extract::Path(id): axum::extract::Path<u64>,
) -> (StatusCode, Json<UserView>) {
  let user = UserView {
    id,
    username: "demo".into(),
    email: "demo@example.com".into(),
  };
  (StatusCode::OK, Json(user))
}

#[utoipa::path(
    put,
    path = "/{id}",
    request_body = UserUpdate,
    responses(
        (status = 200, description = "Update user", body = UserView),
        (status = 404, description = "User not found")
    )
)]
pub async fn update_user(
  State(_state): State<AppState>,
  axum::extract::Path(id): axum::extract::Path<u64>,
  Json(payload): Json<UserUpdate>,
) -> (StatusCode, Json<UserView>) {
  let user = UserView {
    id,
    username: payload.username.unwrap_or("demo".into()),
    email: payload.email.unwrap_or("demo@example.com".into()),
  };
  (StatusCode::OK, Json(user))
}

#[utoipa::path(
  delete,
  path = "/{id}",
  responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found")
  )
)]
pub async fn delete_user(
  State(_state): State<AppState>,
  axum::extract::Path(_id): axum::extract::Path<u64>,
) -> StatusCode {
  StatusCode::NO_CONTENT
}

#[utoipa::path(
  get,
  path = "/user",
  responses(
        (status = 200, description = "List users", body = [UserView])
  )
)]
pub async fn list_users(State(_state): State<AppState>) -> Json<Vec<UserView>> {
  let users = vec![
    UserView {
      id: 1,
      username: "demo1".into(),
      email: "demo1@example.com".into(),
    },
    UserView {
      id: 2,
      username: "demo2".into(),
      email: "demo2@example.com".into(),
    },
  ];
  Json(users)
}

// 子路由
pub fn user_routes() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes![create_user, list_users])
    .routes(routes![get_user])
    .routes(routes![delete_user])
    .routes(routes![update_user])
}
