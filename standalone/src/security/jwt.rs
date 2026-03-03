use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
  pub sub: String,
  pub org: Option<String>,
  pub iat: usize,
  pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvitationClaims {
  pub sub: String,
  pub org: String,
  pub email: String,
  pub iat: usize,
  pub exp: usize,
}

pub fn issue_access_token(
  secret: &str,
  user_id: &str,
  organization_id: Option<&str>,
) -> Result<String, jsonwebtoken::errors::Error> {
  let now = Utc::now();
  let exp = now + Duration::hours(24);

  let claims = AccessClaims {
    sub: user_id.to_string(),
    org: organization_id.map(ToString::to_string),
    iat: now.timestamp() as usize,
    exp: exp.timestamp() as usize,
  };

  encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
  )
}

pub fn verify_access_token(
  secret: &str,
  token: &str,
) -> Result<AccessClaims, jsonwebtoken::errors::Error> {
  let data = decode::<AccessClaims>(
    token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &Validation::default(),
  )?;
  Ok(data.claims)
}

pub fn issue_invitation_token(
  secret: &str,
  invitation_id: &str,
  organization_id: &str,
  email: &str,
  expires_at: chrono::DateTime<chrono::Utc>,
) -> Result<String, jsonwebtoken::errors::Error> {
  let now = Utc::now();
  let claims = InvitationClaims {
    sub: invitation_id.to_string(),
    org: organization_id.to_string(),
    email: email.to_string(),
    iat: now.timestamp() as usize,
    exp: expires_at.timestamp() as usize,
  };

  encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
  )
}

pub fn verify_invitation_token(
  secret: &str,
  token: &str,
) -> Result<InvitationClaims, jsonwebtoken::errors::Error> {
  let data = decode::<InvitationClaims>(
    token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &Validation::default(),
  )?;
  Ok(data.claims)
}
