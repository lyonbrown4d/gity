use gity_entities::{project_access_tokens, project_deploy_keys};
use serde_json::{Value, json};
pub(super) fn token_json(token: &project_access_tokens::Model) -> Value {
  json!({
    "id": token.id,
    "project_id": token.project_id,
    "kind": token.kind,
    "name": token.name,
    "username": token.username,
    "token": token.token,
    "scopes": token.scopes,
    "created_by_user_id": token.created_by_user_id,
    "expires_at": token.expires_at,
    "revoked_at": token.revoked_at,
    "last_used_at": token.last_used_at,
    "created_at": token.created_at,
    "updated_at": token.updated_at,
  })
}

pub(super) fn deploy_key_json(key: &project_deploy_keys::Model) -> Value {
  json!({
    "id": key.id,
    "project_id": key.project_id,
    "title": key.title,
    "fingerprint": key.fingerprint,
    "public_key": key.public_key,
    "can_push": key.can_push,
    "created_by_user_id": key.created_by_user_id,
    "last_used_at": key.last_used_at,
    "created_at": key.created_at,
    "updated_at": key.updated_at,
  })
}
