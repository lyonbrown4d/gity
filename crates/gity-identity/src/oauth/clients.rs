use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use gity_entities::{oauth_clients, oauth_pending_requests};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use url::Url;
use uuid::Uuid;

use super::protocol::{expiry, parse_scopes, random_secret, validate_redirect};
use super::repository::{AuthorizationRequest, ClientRegistration};
use super::{OAuthError, OAuthService};

impl OAuthService {
  pub async fn register_client(
    &self,
    input: ClientRegistration,
  ) -> Result<serde_json::Value, OAuthError> {
    if input.redirect_uris.is_empty()
      || input.redirect_uris.len() > 20
      || input.client_name.trim().is_empty()
      || input.client_name.len() > 128
    {
      return Err(OAuthError::Invalid);
    }
    for uri in &input.redirect_uris {
      if uri.len() > 2048 {
        return Err(OAuthError::Invalid);
      }
      validate_redirect(uri)?;
    }
    let count = oauth_clients::Entity::find()
      .filter(oauth_clients::Column::IsDynamic.eq(true))
      .all(&self.db)
      .await?
      .len();
    if count >= self.config.max_dynamic_clients {
      return Err(OAuthError::Invalid);
    }
    let client_id = random_secret(32);
    oauth_clients::ActiveModel {
      id: Set(gity_core::new_id()),
      client_id: Set(client_id.clone()),
      name: Set(input.client_name.trim().to_owned()),
      redirect_uris: Set(
        serde_json::to_value(&input.redirect_uris).map_err(|_| OAuthError::Invalid)?,
      ),
      created_by: Set(None),
      is_dynamic: Set(true),
      created_at: Set(Utc::now().naive_utc()),
    }
    .insert(&self.db)
    .await?;
    Ok(
      serde_json::json!({"client_id": client_id, "client_name": input.client_name, "redirect_uris": input.redirect_uris, "token_endpoint_auth_method": "none", "grant_types": ["authorization_code", "refresh_token"], "response_types": ["code"]}),
    )
  }

  pub async fn begin_authorization(
    &self,
    request: AuthorizationRequest,
  ) -> Result<String, OAuthError> {
    if request.response_type != "code"
      || request.client_id.is_empty()
      || request.client_id.len() > 128
      || request.redirect_uri.len() > 2048
      || request.scope.len() > 256
      || request
        .nonce
        .as_ref()
        .is_some_and(|nonce| nonce.len() > 512)
      || request.code_challenge_method != "S256"
      || request.state.len() < 16
      || request.state.len() > 512
      || request.code_challenge.len() != 43
      || request.resource != self.resource
    {
      return Err(OAuthError::Invalid);
    }
    let verifier_check = URL_SAFE_NO_PAD
      .decode(&request.code_challenge)
      .map_err(|_| OAuthError::Invalid)?;
    if verifier_check.len() != 32 {
      return Err(OAuthError::Invalid);
    }
    validate_redirect(&request.redirect_uri)?;
    let client = oauth_clients::Entity::find()
      .filter(oauth_clients::Column::ClientId.eq(&request.client_id))
      .one(&self.db)
      .await?
      .ok_or(OAuthError::Invalid)?;
    let redirects: Vec<String> =
      serde_json::from_value(client.redirect_uris).map_err(|_| OAuthError::Invalid)?;
    if !redirects.iter().any(|uri| uri == &request.redirect_uri) {
      return Err(OAuthError::Invalid);
    }
    let scopes = parse_scopes(&request.scope)?;
    let request_key = Uuid::now_v7().to_string();
    oauth_pending_requests::ActiveModel {
      id: Set(gity_core::new_id()),
      request_key: Set(request_key.clone()),
      client_id: Set(request.client_id),
      user_id: Set(None),
      redirect_uri: Set(request.redirect_uri),
      code_challenge: Set(request.code_challenge),
      scopes: Set(scopes.join(" ")),
      state: Set(request.state),
      resource: Set(request.resource),
      nonce: Set(request.nonce),
      admin_mode: Set(scopes.iter().any(|s| s == "admin_mode")),
      status: Set("pending".into()),
      expires_at: Set(expiry(self.config.pending_request_ttl_seconds)),
    }
    .insert(&self.db)
    .await?;
    let mut target = Url::parse(&self.issuer).map_err(|_| OAuthError::Configuration)?;
    target.set_path("/oauth/consent");
    target
      .query_pairs_mut()
      .append_pair("request_id", &request_key);
    Ok(target.to_string())
  }
}
