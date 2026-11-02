use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{NaiveDateTime, Utc};
use gity_entities::{oauth_authorization_codes, oauth_token_families, oauth_tokens, users};
use rand::Rng;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{OAuthError, OAuthService, TokenReply, TokenRequest};

#[derive(Clone, Debug)]
pub struct OAuthPrincipal {
  pub user_id: i64,
  pub username: String,
  pub is_super_admin: bool,
  pub scopes: Vec<String>,
}

impl OAuthService {
  pub async fn token(&self, request: TokenRequest) -> Result<TokenReply, OAuthError> {
    if request.client_id.is_empty()
      || request.client_id.len() > 128
      || request
        .resource
        .as_ref()
        .is_some_and(|resource| resource.len() > 2048)
      || request.code.as_ref().is_some_and(|code| code.len() > 128)
      || request
        .refresh_token
        .as_ref()
        .is_some_and(|token| token.len() > 512)
      || request
        .redirect_uri
        .as_ref()
        .is_some_and(|uri| uri.len() > 2048)
      || request
        .code_verifier
        .as_ref()
        .is_some_and(|verifier| verifier.len() > 128)
    {
      return Err(OAuthError::Invalid);
    }
    if request
      .resource
      .as_deref()
      .is_some_and(|r| r != self.resource)
    {
      return Err(OAuthError::Invalid);
    }
    match request.grant_type.as_str() {
      "authorization_code" => self.exchange_code(request).await,
      "refresh_token" => self.rotate_refresh(request).await,
      _ => Err(OAuthError::Invalid),
    }
  }

  async fn exchange_code(&self, request: TokenRequest) -> Result<TokenReply, OAuthError> {
    let code = request.code.ok_or(OAuthError::Invalid)?;
    let redirect = request.redirect_uri.ok_or(OAuthError::Invalid)?;
    let verifier = request.code_verifier.ok_or(OAuthError::Invalid)?;
    if !(43..=128).contains(&verifier.len())
      || !verifier
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"-._~".contains(&b))
    {
      return Err(OAuthError::Invalid);
    }
    let hash = digest(&code);
    let row = oauth_authorization_codes::Entity::find()
      .filter(oauth_authorization_codes::Column::CodeHash.eq(hash))
      .one(&self.db)
      .await?
      .ok_or(OAuthError::Invalid)?;
    if row.client_id != request.client_id
      || row.redirect_uri != redirect
      || row.expires_at <= Utc::now().naive_utc()
      || row.redeemed_at.is_some()
      || !pkce_matches(&verifier, &row.code_challenge)
    {
      return Err(OAuthError::Invalid);
    }
    let user = users::Entity::find_by_id(row.user_id)
      .one(&self.db)
      .await?
      .ok_or(OAuthError::Invalid)?;
    if user.status != "active" || (row.admin_mode && user.is_super_admin != 1) {
      return Err(OAuthError::Invalid);
    }
    super::oxide_machine::issue_from_code(self, row, &code).await
  }
}

pub(super) async fn create_pair(
  tx: &sea_orm::DatabaseTransaction,
  client_id: &str,
  user_id: i64,
  scopes: &str,
  resource: &str,
  admin_mode: bool,
  config: &gity_core::OAuthConfig,
) -> Result<TokenReply, OAuthError> {
  let access = format!("gity_oauth_{}", secret());
  let refresh = format!("gity_oauth_{}", secret());
  let family = Uuid::now_v7().to_string();
  oauth_token_families::ActiveModel {
    id: Set(gity_core::new_id()),
    family_key: Set(family.clone()),
    created_at: Set(Utc::now().naive_utc()),
    revoked_at: Set(None),
  }
  .insert(tx)
  .await?;
  for (raw, kind, ttl) in [
    (&access, "access", config.access_token_ttl_seconds),
    (&refresh, "refresh", config.refresh_token_ttl_seconds),
  ] {
    oauth_tokens::ActiveModel {
      id: Set(gity_core::new_id()),
      token_hash: Set(digest(raw)),
      family_key: Set(family.clone()),
      client_id: Set(client_id.to_owned()),
      user_id: Set(user_id),
      kind: Set(kind.into()),
      scopes: Set(scopes.to_owned()),
      resource: Set(resource.to_owned()),
      admin_mode: Set(admin_mode),
      expires_at: Set(expiry(ttl)),
      revoked_at: Set(None),
      replaced_by_hash: Set(None),
    }
    .insert(tx)
    .await?;
  }
  Ok(TokenReply {
    access_token: access,
    token_type: "Bearer",
    expires_in: config.access_token_ttl_seconds,
    refresh_token: refresh,
    scope: scopes.to_owned(),
  })
}

fn pkce_matches(verifier: &str, expected: &str) -> bool {
  URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())) == expected
}
pub(super) fn secret() -> String {
  let mut bytes = [0u8; 32];
  rand::rng().fill_bytes(&mut bytes);
  URL_SAFE_NO_PAD.encode(bytes)
}
pub(super) fn digest(value: &str) -> String {
  format!("{:x}", Sha256::digest(value.as_bytes()))
}
pub(super) fn expiry(seconds: u64) -> NaiveDateTime {
  (Utc::now() + chrono::Duration::seconds(seconds as i64)).naive_utc()
}
