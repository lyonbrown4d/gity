use crate::http::app_state::AppState;
use crate::security::jwt::issue_access_token;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use domain::user::CreateUser;
use entity::{organization_members, organizations, users};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Condition, DbErr, EntityTrait, QueryFilter, QueryOrder,
  TransactionTrait,
};
use sea_orm::Set;
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
}

#[utoipa::path(
  post,
  path = "/register",
  request_body = RegisterRequest,
  responses(
    (status = 201, description = "User and default organization created", body = AuthResponse),
    (status = 409, description = "User or organization already exists", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn register(
  State(state): State<AppState>,
  Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<ErrorResponse>)> {
  let txn = match state.db_conn.begin().await {
    Ok(txn) => txn,
    Err(err) => return Err(internal_error(err)),
  };

  let duplicated_user = match users::Entity::find()
    .filter(
      Condition::any()
        .add(users::Column::Username.eq(payload.username.clone()))
        .add(users::Column::Email.eq(payload.email.clone())),
    )
    .one(&txn)
    .await
  {
    Ok(user) => user.is_some(),
    Err(err) => return Err(internal_error(err)),
  };

  if duplicated_user {
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorResponse {
        message: "username or email already exists".to_string(),
      }),
    ));
  }

  let create_user = CreateUser {
    username: payload.username.clone(),
    email: payload.email.clone(),
    password: payload.password.clone(),
  };

  let user = match users::ActiveModel::from(create_user).insert(&txn).await {
    Ok(user) => user,
    Err(err) => return Err(map_db_error(err, "failed to create user")),
  };

  let org_name = payload
    .organization_name
    .clone()
    .unwrap_or_else(|| format!("{}'s organization", payload.username));
  let org_key = payload
    .organization_key
    .clone()
    .unwrap_or_else(|| default_org_key(&payload.username));

  let org_active = organizations::ActiveModel {
    key: Set(org_key),
    name: Set(org_name),
    status: Set(organizations::OrgStatus::Active),
    ..Default::default()
  };

  let organization = match org_active.insert(&txn).await {
    Ok(org) => org,
    Err(err) => return Err(map_db_error(err, "failed to create organization")),
  };

  let member_active = organization_members::ActiveModel {
    organization_id: Set(organization.id.clone()),
    user_id: Set(user.id.clone()),
    role: Set(organization_members::MemberRole::Owner),
    ..Default::default()
  };

  if let Err(err) = member_active.insert(&txn).await {
    return Err(map_db_error(err, "failed to create organization membership"));
  }

  if let Err(err) = txn.commit().await {
    return Err(internal_error(err));
  }

  let token = match issue_token(&state, &user.id, Some(&organization.id)) {
    Ok(token) => token,
    Err(err) => {
      return Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          message: format!("failed to issue token: {err}"),
        }),
      ));
    }
  };

  Ok((
    StatusCode::CREATED,
    Json(AuthResponse {
      user_id: user.id,
      username: user.username,
      organization_id: Some(organization.id),
      organization_name: Some(organization.name),
      token,
    }),
  ))
}

#[utoipa::path(
  post,
  path = "/login",
  request_body = LoginRequest,
  responses(
    (status = 200, description = "Login success", body = AuthResponse),
    (status = 401, description = "Invalid credentials", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn login(
  State(state): State<AppState>,
  Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<ErrorResponse>)> {
  let user = match users::Entity::find()
    .filter(
      Condition::any()
        .add(users::Column::Username.eq(payload.username.clone()))
        .add(users::Column::Email.eq(payload.username.clone())),
    )
    .one(&state.db_conn)
    .await
  {
    Ok(Some(user)) => user,
    Ok(None) => {
      return Err((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
          message: "invalid username/email or password".to_string(),
        }),
      ));
    }
    Err(err) => return Err(internal_error(err)),
  };

  if !users::Model::verify_password(payload.password.as_str(), user.password.as_str()) {
    return Err((
      StatusCode::UNAUTHORIZED,
      Json(ErrorResponse {
        message: "invalid username/email or password".to_string(),
      }),
    ));
  }

  let membership: Option<organization_members::Model> = match organization_members::Entity::find()
    .filter(
      Condition::all()
        .add(organization_members::Column::UserId.eq(user.id.clone()))
        .add(organization_members::Column::DeletedAt.is_null()),
    )
    .order_by_asc(organization_members::Column::CreatedAt)
    .one(&state.db_conn)
    .await
  {
    Ok(member) => member,
    Err(err) => return Err(internal_error(err)),
  };

  let organization = match membership.as_ref() {
    Some(member) => match organizations::Entity::find_by_id(member.organization_id.clone())
      .filter(organizations::Column::DeletedAt.is_null())
      .one(&state.db_conn)
      .await
    {
      Ok(org) => org,
      Err(err) => return Err(internal_error(err)),
    },
    None => None,
  };

  let token = match issue_token(
    &state,
    &user.id,
    organization.as_ref().map(|org| org.id.as_str()),
  ) {
    Ok(token) => token,
    Err(err) => {
      return Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          message: format!("failed to issue token: {err}"),
        }),
      ));
    }
  };

  Ok((
    StatusCode::OK,
    Json(AuthResponse {
      user_id: user.id,
      username: user.username,
      organization_id: organization.as_ref().map(|org| org.id.clone()),
      organization_name: organization.as_ref().map(|org| org.name.clone()),
      token,
    }),
  ))
}

pub fn auth_routes() -> OpenApiRouter<AppState> {
  OpenApiRouter::new().routes(routes![register, login])
}

fn default_org_key(username: &str) -> String {
  let normalized = username.trim().to_lowercase();
  let mapped: String = normalized
    .chars()
    .map(|c| {
      if c.is_ascii_alphanumeric() {
        c
      } else {
        '-'
      }
    })
    .collect();

  format!("{}-org", collapse_hyphens(mapped))
}

fn collapse_hyphens(input: String) -> String {
  input
    .split('-')
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join("-")
}

fn issue_token(
  state: &AppState,
  user_id: &str,
  organization_id: Option<&str>,
) -> Result<String, String> {
  let secret = state
    .config
    .auth
    .as_ref()
    .and_then(|auth| auth.jwt_secret.clone())
    .filter(|secret| !secret.is_empty())
    .unwrap_or_else(|| "gity-dev-secret-change-me".to_string());

  issue_access_token(secret.as_str(), user_id, organization_id).map_err(|err| err.to_string())
}

fn map_db_error(err: DbErr, message: &str) -> (StatusCode, Json<ErrorResponse>) {
  if is_unique_violation(&err) {
    (
      StatusCode::CONFLICT,
      Json(ErrorResponse {
        message: err.to_string(),
      }),
    )
  } else {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse {
        message: format!("{message}: {err}"),
      }),
    )
  }
}

fn internal_error(err: DbErr) -> (StatusCode, Json<ErrorResponse>) {
  (
    StatusCode::INTERNAL_SERVER_ERROR,
    Json(ErrorResponse {
      message: format!("internal server error: {err}"),
    }),
  )
}

fn is_unique_violation(err: &DbErr) -> bool {
  let message = err.to_string().to_lowercase();
  message.contains("unique") || message.contains("duplicate")
}
