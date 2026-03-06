use crate::configuration::cfg::Config;
use async_trait::async_trait;
use domain::user::CreateUser;
use entity::users;
use repository::UsersRepository;
use sea_orm::DbErr;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CredentialLogin {
  pub username: String,
  pub password: String,
}

#[derive(Debug)]
pub enum AuthenticationError {
  Internal(String),
}

#[derive(Debug)]
pub enum AuthenticationDecision {
  Skip,
  Reject,
  Authenticated(users::Model),
}

#[async_trait]
pub trait AuthenticationProvider: Send + Sync {
  async fn authenticate(
    &self,
    credentials: &CredentialLogin,
  ) -> Result<AuthenticationDecision, AuthenticationError>;
}

#[derive(Clone)]
pub struct AuthenticationManager {
  providers: Vec<Arc<dyn AuthenticationProvider>>,
}

impl AuthenticationManager {
  pub fn new(providers: Vec<Arc<dyn AuthenticationProvider>>) -> Self {
    Self { providers }
  }

  pub async fn authenticate(
    &self,
    credentials: &CredentialLogin,
  ) -> Result<Option<users::Model>, AuthenticationError> {
    for provider in &self.providers {
      match provider.authenticate(credentials).await? {
        AuthenticationDecision::Skip => continue,
        AuthenticationDecision::Reject => continue,
        AuthenticationDecision::Authenticated(user) => return Ok(Some(user)),
      }
    }

    Ok(None)
  }
}

#[derive(Debug, Clone)]
struct SuperAdminCredential {
  username: String,
  password: String,
}

pub struct SuperAdminCredentialsProvider {
  users_repository: UsersRepository,
  credential: Option<SuperAdminCredential>,
}

impl SuperAdminCredentialsProvider {
  pub fn new(config: &Config, users_repository: UsersRepository) -> Self {
    Self {
      users_repository,
      credential: extract_super_admin_credential(config),
    }
  }

  async fn ensure_local_super_admin_user(
    &self,
    credential: &SuperAdminCredential,
  ) -> Result<users::Model, AuthenticationError> {
    let identity = credential.username.trim();
    let existing = self
      .users_repository
      .find_user_by_username_or_email(identity)
      .await
      .map_err(map_db_error)?;
    if let Some(user) = existing {
      if is_active_user(&user) {
        return Ok(user);
      }
      return Err(AuthenticationError::Internal(
        "super admin user exists but is not active".to_string(),
      ));
    }

    let email = synthetic_email(identity);
    let create = CreateUser {
      username: identity.to_string(),
      email,
      password: credential.password.clone(),
    };
    let inserted = self
      .users_repository
      .insert_user(users::ActiveModel::from(create))
      .await
      .map_err(map_db_error)?;
    Ok(inserted)
  }
}

#[async_trait]
impl AuthenticationProvider for SuperAdminCredentialsProvider {
  async fn authenticate(
    &self,
    credentials: &CredentialLogin,
  ) -> Result<AuthenticationDecision, AuthenticationError> {
    let Some(admin) = self.credential.as_ref() else {
      return Ok(AuthenticationDecision::Skip);
    };

    if normalize_identity(credentials.username.as_str())
      != normalize_identity(admin.username.as_str())
    {
      return Ok(AuthenticationDecision::Skip);
    }

    if credentials.password != admin.password {
      return Ok(AuthenticationDecision::Reject);
    }

    let user = self.ensure_local_super_admin_user(admin).await?;
    Ok(AuthenticationDecision::Authenticated(user))
  }
}

pub struct DbUserPasswordProvider {
  users_repository: UsersRepository,
}

impl DbUserPasswordProvider {
  pub fn new(users_repository: UsersRepository) -> Self {
    Self { users_repository }
  }
}

#[async_trait]
impl AuthenticationProvider for DbUserPasswordProvider {
  async fn authenticate(
    &self,
    credentials: &CredentialLogin,
  ) -> Result<AuthenticationDecision, AuthenticationError> {
    let Some(user) = self
      .users_repository
      .find_user_by_username_or_email(credentials.username.as_str())
      .await
      .map_err(map_db_error)?
    else {
      return Ok(AuthenticationDecision::Reject);
    };

    if !is_active_user(&user) {
      return Ok(AuthenticationDecision::Reject);
    }

    if !users::Model::verify_password(credentials.password.as_str(), user.password.as_str()) {
      return Ok(AuthenticationDecision::Reject);
    }

    Ok(AuthenticationDecision::Authenticated(user))
  }
}

fn extract_super_admin_credential(config: &Config) -> Option<SuperAdminCredential> {
  let auth = config.auth.as_ref()?;
  let admin = auth.admin.as_ref()?;
  let username = admin.username.as_ref()?.trim().to_string();
  let password = admin.password.as_ref()?.trim().to_string();
  if username.is_empty() || password.is_empty() {
    return None;
  }

  Some(SuperAdminCredential { username, password })
}

fn normalize_identity(value: &str) -> String {
  value.trim().to_ascii_lowercase()
}

fn sanitize_email_local(local: &str) -> String {
  let sanitized = local
    .chars()
    .map(|c| {
      if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
        c.to_ascii_lowercase()
      } else {
        '-'
      }
    })
    .collect::<String>()
    .trim_matches('-')
    .to_string();

  if sanitized.is_empty() {
    "super-admin".to_string()
  } else {
    sanitized
  }
}

fn synthetic_email(identity: &str) -> String {
  let normalized = identity.trim().to_ascii_lowercase();
  if normalized.contains('@') {
    normalized
  } else {
    format!("{}@local.gity", sanitize_email_local(normalized.as_str()))
  }
}

fn is_active_user(user: &users::Model) -> bool {
  user.deleted_at.is_none() && user.status == users::UserStatus::Active
}

fn map_db_error(err: DbErr) -> AuthenticationError {
  AuthenticationError::Internal(format!("authentication provider database error: {err}"))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::configuration::cfg::{Admin, Auth};

  #[test]
  fn extract_super_admin_credential_reads_auth_admin_fields() {
    let mut cfg = Config::default();
    cfg.auth = Some(Auth {
      enable_jwt: None,
      jwt_secret: None,
      enable_ldap: None,
      ldap_url: None,
      super_admins: None,
      admin: Some(Admin {
        username: Some("admin".to_string()),
        password: Some("secret".to_string()),
      }),
    });

    let credential = extract_super_admin_credential(&cfg).expect("credential should be present");
    assert_eq!(credential.username, "admin");
    assert_eq!(credential.password, "secret");
  }

  #[test]
  fn synthetic_email_uses_identity_when_already_email() {
    assert_eq!(synthetic_email("Root@Example.com"), "root@example.com");
  }

  #[test]
  fn synthetic_email_generates_local_email_for_username() {
    assert_eq!(synthetic_email("Root User"), "root-user@local.gity");
  }
}
