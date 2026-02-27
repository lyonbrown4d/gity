use crate::http::app_state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router};
use entity::users;
use jsonwebtoken::{EncodingKey, Header, encode};
use sea_orm::ColumnTrait;
use sea_orm::{EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
  pub username: String,
  pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
  pub username: String,
  pub token: String,
}
#[utoipa::path(
  post, path = "/login",
  request_body = LoginRequest,
  responses(
        (status = 200, description = "Login success", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    )
)
]
pub async fn login(
  State(_state): State<AppState>,
  Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<LoginResponse>) {
  // let user = users::Entity::find()
  //   .filter(users::Column::Username.eq(payload.username.clone()))
  //   .one(&_state.db_conn)
  //   .await
  //   .unwrap();
  // 
  // let user = match user {
  //   Some(u) => u,
  //   None => {
  //     return (
  //       StatusCode::UNAUTHORIZED,
  //       Json(LoginResponse {
  //         username: "".to_string(),
  //         token: "".to_string(),
  //       }),
  //     );
  //   }
  // };
  // 
  // let valid = users::Model::verify_password(payload.username.as_str(), payload.password.as_str());
  // let valid = true;
  // if !valid {
  //   return (
  //     StatusCode::UNAUTHORIZED,
  //     Json(LoginResponse {
  //       username: "".to_string(),
  //       token: "".to_string(),
  //     }),
  //   );
  // }
  // 
  // // 生成 JWT
  let token = encode_jwt("&user.username"); // 自己实现生成 token 的函数
  // 
  // // 返回结果
  let resp = LoginResponse {
    username: "123".parse().unwrap(),
    token,
  };
  (StatusCode::OK, Json(resp))
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  sub: String,
  exp: usize,
}

pub fn encode_jwt(username: &str) -> String {
  let exp = SystemTime::now()
    .checked_add(Duration::from_secs(60 * 60)) // 1小时有效期
    .unwrap()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() as usize;

  let claims = Claims {
    sub: username.to_string(),
    exp,
  };

  let key = "secret_key"; // 生产环境请用安全方式管理
  encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(key.as_bytes()),
  )
  .unwrap()
}

// pub fn auth_routes() -> OpenApiRouter<AppState> {
//   OpenApiRouter::new().routes(routes![login])
// }
// 
