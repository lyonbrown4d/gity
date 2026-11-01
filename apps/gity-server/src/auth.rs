use crate::AppState;
mod challenge;
mod sessions;
use crate::openapi::{schemas as api, user_schemas as spec};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use axum::{Json, Router, extract::State, http::HeaderMap};
pub(crate) use challenge::basic_challenge;
use gity_core::{AppError, Principal, new_id, now, ok};
use sea_orm::ConnectionTrait;
use serde::Deserialize;
use serde_json::{Value, json};
use sessions::{create_account, session};
use sha2::{Digest, Sha256};

pub fn routes() -> Router<AppState> {
    documented_routes().split_for_parts().0
}
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
    utoipa_axum::router::OpenApiRouter::new()
        .routes(utoipa_axum::routes!(login))
        .routes(utoipa_axum::routes!(register))
        .routes(utoipa_axum::routes!(refresh))
        .routes(utoipa_axum::routes!(logout))
}
pub fn token_hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
pub fn secret() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
pub fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}
pub async fn principal(state: &AppState, headers: &HeaderMap) -> Result<Principal, AppError> {
    let token = bearer(headers).ok_or_else(|| AppError::unauthorized("authentication required"))?;
    principal_token(state, token).await
}
pub async fn principal_token(state: &AppState, token: &str) -> Result<Principal, AppError> {
    if let Some(row) = state.db.one("SELECT u.* FROM auth_sessions s JOIN users u ON u.id=s.user_id WHERE s.token_hash=? AND s.kind='access' AND s.expires_at>?",vec![token_hash(token).into(),now().into()]).await? { return from_user(&row); }
    // Personal tokens are stored only as digests. Refresh tokens never authenticate requests.
    if let Some(row) = state.db.one("SELECT u.* FROM user_access_tokens t JOIN users u ON u.id=t.user_id WHERE t.token=? AND t.name NOT IN ('access','refresh')",vec![format!("sha256:{}",token_hash(token)).into()]).await? { return from_user(&row); }
    Err(AppError::unauthorized("invalid or expired credentials"))
}
fn from_user(row: &Value) -> Result<Principal, AppError> {
    if row["status"].as_str() != Some("active") {
        return Err(AppError::unauthorized("invalid or expired credentials"));
    }
    Ok(Principal {
        user_id: row["id"]
            .as_i64()
            .ok_or_else(|| AppError::internal("invalid user identifier"))?,
        username: row["username"].as_str().unwrap_or_default().into(),
        is_super_admin: row["is_super_admin"]
            .as_bool()
            .unwrap_or_else(|| row["is_super_admin"].as_i64() == Some(1)),
    })
}
#[derive(Deserialize, utoipa::ToSchema)]
struct Credentials {
    username: String,
    #[schema(format=Password,write_only,min_length=8,max_length=1024)]
    password: String,
}
#[derive(Deserialize, utoipa::ToSchema)]
struct Refresh {
    refresh_token: String,
}
#[utoipa::path(
    post,path="/auth/login",tag="auth",
    description="Authenticates a password; an unknown username creates an account, preserving the existing login workflow.",
    request_body=Credentials,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Session>),api::Errors),
    security((), ("bearer"=[]))
)]
async fn login(
    State(state): State<AppState>,
    Json(body): Json<Credentials>,
) -> Result<Json<Value>, AppError> {
    validate_credentials(&body)?;
    let user = state
        .db
        .one(
            "SELECT * FROM users WHERE username=?",
            vec![body.username.trim().into()],
        )
        .await?;
    let Some(user) = user else {
        return create_account(&state, body).await;
    };
    let id = user["id"]
        .as_i64()
        .ok_or_else(|| AppError::internal("invalid user"))?;
    let row = state
        .db
        .one(
            "SELECT password_hash FROM user_passwords WHERE user_id=?",
            vec![id.into()],
        )
        .await?;
    let encoded = row
        .as_ref()
        .and_then(|r| r["password_hash"].as_str())
        .ok_or_else(|| {
            AppError::unauthorized("password not configured; administrator must run set-password")
        })?
        .to_owned();
    let verified = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&encoded).is_ok_and(|hash| {
            Argon2::default()
                .verify_password(body.password.as_bytes(), &hash)
                .is_ok()
        })
    })
    .await
    .map_err(|_| AppError::internal("password verification failed"))?;
    if !verified {
        return Err(AppError::unauthorized("invalid username or password"));
    }
    session(&state, &user, None).await
}
#[utoipa::path(
    post,path="/auth/register",tag="auth",
    description="Creates a password-backed account and personal organization.",
    request_body=Credentials,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Session>),api::Errors),
    security((), ("bearer"=[]))
)]
async fn register(
    State(state): State<AppState>,
    Json(body): Json<Credentials>,
) -> Result<Json<Value>, AppError> {
    validate_credentials(&body)?;
    create_account(&state, body).await
}
pub(crate) fn validate_username(raw: &str) -> Result<&str, AppError> {
    let username = raw.trim();
    if username.is_empty()
        || username.len() > 64
        || !username
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
    {
        return Err(AppError::bad_request(
            "username must use 1–64 letters, digits, dots, dashes or underscores",
        ));
    }
    Ok(username)
}
fn validate_credentials(body: &Credentials) -> Result<(), AppError> {
    validate_username(&body.username)?;
    if body.password.len() < 8 || body.password.len() > 1024 {
        return Err(AppError::bad_request("password must contain 8–1024 bytes"));
    }
    Ok(())
}
pub async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|v| v.to_string())
            .map_err(|_| AppError::internal("password hashing failed"))
    })
    .await
    .map_err(|_| AppError::internal("password hashing failed"))?
}
#[utoipa::path(
    post,path="/auth/refresh",tag="auth",
    description="Rotates a one-use refresh token; replay fails with 401.",
    request_body=Refresh,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Session>),api::Errors),
    security((), ("bearer"=[]))
)]
async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<Refresh>,
) -> Result<Json<Value>, AppError> {
    let hash = token_hash(&body.refresh_token);
    let row=state.db.one("SELECT s.family,u.* FROM auth_sessions s JOIN users u ON u.id=s.user_id WHERE s.token_hash=? AND s.kind='refresh' AND s.expires_at>?",vec![hash.clone().into(),now().into()]).await?.ok_or_else(||AppError::unauthorized("invalid refresh token"))?;
    if state
        .db
        .execute(
            "DELETE FROM auth_sessions WHERE token_hash=? AND kind='refresh'",
            vec![hash.into()],
        )
        .await?
        != 1
    {
        return Err(AppError::unauthorized("refresh token already used"));
    }
    session(&state, &row, row["family"].as_str().map(str::to_owned)).await
}
#[utoipa::path(
    post,path="/auth/logout",tag="auth",
    description="Revokes the session family referenced by bearer or refresh credentials.",
    request_body=spec::Logout,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::OkResult>),api::Errors),
    security((), ("bearer"=[]))
)]
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    for token in [bearer(&headers), body["refresh_token"].as_str()]
        .into_iter()
        .flatten()
    {
        if let Some(row) = state
            .db
            .one(
                "SELECT family FROM auth_sessions WHERE token_hash=?",
                vec![token_hash(token).into()],
            )
            .await?
        {
            state
                .db
                .execute(
                    "DELETE FROM auth_sessions WHERE family=?",
                    vec![row["family"].as_str().unwrap_or_default().into()],
                )
                .await?;
        }
    }
    Ok(ok(json!({"ok":true})))
}
