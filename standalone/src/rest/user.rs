use crate::app_state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use domain::user::{CreateUser, UserViewObject};
use entity::users;
use sea_orm::ActiveModelTrait;

#[utoipa::path(get, path = "/api/v1/user", responses((status = OK)))]
pub async fn root(
  State(state): State<AppState>,
) -> &'static str {
  "Hello, World!"
}

#[utoipa::path(post, path = "/api/v1/user", responses((status = OK)))]
pub async fn create_user(
  State(state): State<AppState>,
  Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<UserViewObject>) {
  let active_user: users::ActiveModel = payload.into();
  let inserted_user = active_user.insert(&state.db_conn).await.unwrap();
  let view_user: UserViewObject = inserted_user.into();
  (StatusCode::CREATED, Json(view_user))
}
