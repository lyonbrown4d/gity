use std::time::Duration;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{NaiveDateTime, Utc};
use gity_core::OAuthConfig;
use rand::Rng;
use sea_orm::DatabaseConnection;
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
  #[error("invalid OAuth request")]
  Invalid,
  #[error("OAuth request expired or already used")]
  Expired,
  #[error("OAuth persistence failed")]
  Database(#[from] sea_orm::DbErr),
  #[error("OAuth service configuration is invalid")]
  Configuration,
}

#[derive(Clone)]
pub struct OAuthService {
  pub(super) db: DatabaseConnection,
  pub(super) config: OAuthConfig,
  pub(super) issuer: String,
  pub(super) resource: String,
}

impl OAuthService {
  pub fn new(
    db: DatabaseConnection,
    config: OAuthConfig,
    issuer: String,
  ) -> Result<Self, OAuthError> {
    let base = Url::parse(&issuer).map_err(|_| OAuthError::Configuration)?;
    if !(base.scheme() == "https" || (base.scheme() == "http" && is_loopback(base.host_str())))
      || base.host_str().is_none()
      || !base.username().is_empty()
      || base.password().is_some()
      || base.query().is_some()
      || base.fragment().is_some()
      || base.path() != "/"
      || !config.validate()
    {
      return Err(OAuthError::Configuration);
    }
    let resource = format!("{}/mcp", issuer.trim_end_matches('/'));
    Ok(Self {
      db,
      config,
      issuer: issuer.trim_end_matches('/').to_owned(),
      resource,
    })
  }

  pub fn metadata(&self) -> serde_json::Value {
    serde_json::json!({
      "issuer": self.issuer, "authorization_endpoint": format!("{}/oauth/authorize", self.issuer),
      "token_endpoint": format!("{}/oauth/token", self.issuer), "revocation_endpoint": format!("{}/oauth/revoke", self.issuer),
      "registration_endpoint": format!("{}/oauth/register", self.issuer), "response_types_supported": ["code"],
      "grant_types_supported": ["authorization_code", "refresh_token"], "code_challenge_methods_supported": ["S256"],
      "token_endpoint_auth_methods_supported": ["none"], "scopes_supported": ["mcp", "read_api", "write_api", "admin_mode"],
      "resource": self.resource
    })
  }
}

pub(super) fn validate_redirect(value: &str) -> Result<(), OAuthError> {
  let parsed = Url::parse(value).map_err(|_| OAuthError::Invalid)?;
  let valid =
    parsed.scheme() == "https" || (parsed.scheme() == "http" && is_loopback(parsed.host_str()));
  if !valid || parsed.username() != "" || parsed.password().is_some() || parsed.fragment().is_some()
  {
    return Err(OAuthError::Invalid);
  }
  if parsed
    .query_pairs()
    .any(|(key, _)| matches!(key.as_ref(), "code" | "state" | "error"))
  {
    return Err(OAuthError::Invalid);
  }
  Ok(())
}
fn is_loopback(host: Option<&str>) -> bool {
  match host {
    Some("localhost") => true,
    Some(host) => host
      .trim_start_matches('[')
      .trim_end_matches(']')
      .parse::<std::net::IpAddr>()
      .is_ok_and(|address| address.is_loopback()),
    None => false,
  }
}
pub(super) fn parse_scopes(scope: &str) -> Result<Vec<String>, OAuthError> {
  let values: Vec<_> = scope.split_ascii_whitespace().collect();
  let _: oxide_auth::primitives::scope::Scope = scope.parse().map_err(|_| OAuthError::Invalid)?;
  if values.is_empty()
    || values.len() > 4
    || values
      .iter()
      .any(|s| !matches!(*s, "mcp" | "read_api" | "write_api" | "admin_mode"))
  {
    return Err(OAuthError::Invalid);
  }
  if values.iter().filter(|s| **s == "mcp").count() != 1 {
    return Err(OAuthError::Invalid);
  }
  Ok(values.into_iter().map(str::to_owned).collect())
}
pub(super) fn random_secret(bytes: usize) -> String {
  let mut value = vec![0; bytes];
  rand::rng().fill_bytes(&mut value);
  URL_SAFE_NO_PAD.encode(value)
}
pub(super) fn digest(value: &str) -> String {
  format!("{:x}", Sha256::digest(value.as_bytes()))
}
pub(super) fn expiry(seconds: u64) -> NaiveDateTime {
  (Utc::now() + Duration::from_secs(seconds)).naive_utc()
}

#[cfg(test)]
mod tests;
