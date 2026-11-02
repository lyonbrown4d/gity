//! Personal credentials are accessible only through the authenticated account.
use crate::{AppState, auth, openapi::schemas as api};
use axum::{
  Json,
  extract::{Path, State},
  http::HeaderMap,
};
use gity_core::{AppError, ok};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

type Reply = Result<Json<Value>, AppError>;

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateSshKey {
  #[schema(min_length = 1, max_length = 255)]
  title: String,
  #[schema(max_length = 16384)]
  public_key: String,
}
#[derive(Serialize, utoipa::ToSchema)]
pub(super) struct SshKey {
  id: String,
  user_id: String,
  title: String,
  fingerprint: String,
  public_key: String,
  created_at: String,
  updated_at: String,
  last_used_at: Option<String>,
}
fn view(key: gity_entities::user_ssh_keys::Model) -> SshKey {
  let timestamp = |v: chrono::NaiveDateTime| {
    v.and_utc()
      .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
  };
  SshKey {
    id: key.id.to_string(),
    user_id: key.user_id.to_string(),
    title: key.title,
    fingerprint: key.fingerprint,
    public_key: key.public_key,
    created_at: timestamp(key.created_at),
    updated_at: timestamp(key.updated_at),
    last_used_at: key.last_used_at.map(timestamp),
  }
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(get, path="/users/me/ssh-keys", tag="users",
  description="Lists only the authenticated account's public SSH keys, newest first. Administrator privileges do not grant access to another account's keys.",
  responses((status=200, description="Personal SSH keys", body=api::Envelope<Vec<SshKey>>), api::Errors),
  security(("bearer"=[])))]
pub(super) async fn list(State(s): State<AppState>, h: HeaderMap) -> Reply {
  let user = auth::principal(&s, &h).await?;
  Ok(ok(
    s.identity
      .personal_ssh_keys(user.user_id)
      .await?
      .into_iter()
      .map(view)
      .collect::<Vec<_>>(),
  ))
}

#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(post, path="/users/me/ssh-keys", tag="users",
  description="Registers one OpenSSH public key for the authenticated account. Fingerprints are globally unique and cannot also be deploy keys. Private keys and authorized_keys options are rejected.",
  request_body=CreateSshKey,
  responses((status=200, description="SSH key registered", body=api::Envelope<SshKey>), api::Errors),
  security(("bearer"=[])))]
pub(super) async fn create(
  State(s): State<AppState>,
  h: HeaderMap,
  Json(body): Json<CreateSshKey>,
) -> Reply {
  let user = auth::principal(&s, &h).await?;
  let title = body.title.trim();
  if title.is_empty() || title.len() > 255 || title.chars().any(char::is_control) {
    return Err(AppError::bad_request(
      "SSH key title must contain 1–255 bytes without control characters",
    ));
  }
  let (public_key, fingerprint) = crate::ssh::canonical_key(&body.public_key)?;
  if s
    .identity
    .deployment_fingerprint_exists(&fingerprint)
    .await?
    || s.identity.personal_ssh_key(&fingerprint).await?.is_some()
  {
    return Err(AppError::conflict("SSH fingerprint already registered"));
  }
  let command = gity_identity::PersonalSshKeyCommand {
    id: gity_core::new_id(),
    user_id: user.user_id,
    title: title.into(),
    public_key,
    fingerprint,
  };
  let key = s.identity.add_personal_ssh_key(command).await.map_err(|error| {
    if matches!(&error, sea_orm::DbErr::RecordNotFound(_)) {
      AppError::unauthorized("invalid or expired credentials")
    } else if matches!(&error, sea_orm::DbErr::Custom(message) if message == "SSH fingerprint already registered as a deploy key") {
      AppError::conflict("SSH fingerprint already registered")
    } else { AppError::from(error) }
  })?;
  Ok(ok(view(key)))
}

#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(delete, path="/users/me/ssh-keys/{id}", tag="users",
  description="Revokes only a key owned by the authenticated account. New Git commands on existing SSH connections recheck revocation.",
  params(("id"=String, Path, description="Personal SSH key ID")),
  responses((status=200, description="SSH key revoked", body=api::Envelope<crate::openapi::user_schemas::OkResult>), api::Errors),
  security(("bearer"=[])))]
pub(super) async fn remove(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i64>) -> Reply {
  let user = auth::principal(&s, &h).await?;
  if !s.identity.delete_personal_ssh_key(user.user_id, id).await? {
    return Err(AppError::not_found("SSH key not found"));
  }
  Ok(ok(json!({"ok":true})))
}

#[cfg(test)]
#[path = "ssh_keys_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "ssh_key_lifecycle_tests.rs"]
mod lifecycle_tests;
