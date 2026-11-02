use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorizationRequest {
  pub client_id: String,
  pub redirect_uri: String,
  pub response_type: String,
  pub code_challenge: String,
  pub code_challenge_method: String,
  pub scope: String,
  pub state: String,
  pub resource: String,
  pub nonce: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClientRegistration {
  pub client_name: String,
  pub redirect_uris: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PendingAuthorization {
  pub request_id: String,
  pub client_name: String,
  pub scopes: Vec<String>,
  pub resource: String,
  pub requests_admin_mode: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TokenRequest {
  pub grant_type: String,
  pub code: Option<String>,
  pub refresh_token: Option<String>,
  pub client_id: String,
  pub redirect_uri: Option<String>,
  pub code_verifier: Option<String>,
  pub resource: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TokenReply {
  pub access_token: String,
  pub token_type: &'static str,
  pub expires_in: u64,
  pub refresh_token: String,
  pub scope: String,
}
