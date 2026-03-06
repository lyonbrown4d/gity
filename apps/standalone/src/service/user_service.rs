use crate::configuration::cfg::Config;
use domain::user::CreateUser;
use entity::users;
use repository::UsersRepository;
use sea_orm::{DatabaseConnection, DbErr};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct CreateUserInput {
  pub username: String,
  pub email: String,
  pub password: String,
}

#[derive(Debug, Clone)]
pub struct UpdateCurrentUserInput {
  pub username: Option<String>,
  pub email: Option<String>,
  pub password: Option<String>,
}

#[derive(Debug)]
pub enum UserServiceError {
  BadRequest(String),
  Unauthorized(String),
  Forbidden(String),
  NotFound(String),
  Conflict(String),
  Internal(String),
}

#[derive(Clone)]
pub struct UserService {
  db_conn: DatabaseConnection,
  super_admin_identities: HashSet<String>,
}

impl UserService {
  pub fn new(config: &Config, db_conn: DatabaseConnection) -> Self {
    Self {
      db_conn,
      super_admin_identities: collect_super_admin_identities(config),
    }
  }

  pub async fn get_current_user(&self, user_id: &str) -> Result<users::Model, UserServiceError> {
    UsersRepository::find_active_user_by_id(&self.db_conn, user_id)
      .await
      .map_err(Self::internal_error)?
      .ok_or_else(|| UserServiceError::Unauthorized("current user not found".to_string()))
  }

  pub async fn update_current_user(
    &self,
    user_id: &str,
    input: UpdateCurrentUserInput,
  ) -> Result<users::Model, UserServiceError> {
    let mut username = input.username.map(|v| v.trim().to_string());
    let mut email = input.email.map(|v| v.trim().to_ascii_lowercase());

    if username.as_ref().is_some_and(String::is_empty)
      || email.as_ref().is_some_and(String::is_empty)
    {
      return Err(UserServiceError::BadRequest(
        "username/email cannot be empty".to_string(),
      ));
    }

    if input.password.as_ref().is_some_and(|p| p.trim().is_empty()) {
      return Err(UserServiceError::BadRequest(
        "password cannot be empty".to_string(),
      ));
    }

    if username.is_none() && email.is_none() && input.password.is_none() {
      return Err(UserServiceError::BadRequest(
        "at least one field must be provided".to_string(),
      ));
    }

    let current = self.get_current_user(user_id).await?;

    if username
      .as_ref()
      .is_some_and(|value| value == &current.username)
    {
      username = None;
    }
    if email.as_ref().is_some_and(|value| value == &current.email) {
      email = None;
    }

    let (password_hash, password_salt) = match input.password {
      Some(password) => {
        let (hash, salt) = users::Model::hash_password(password.as_str());
        (Some(hash), Some(salt.to_string()))
      }
      None => (None, None),
    };

    UsersRepository::update_user_profile(
      &self.db_conn,
      current,
      username,
      email,
      password_hash,
      password_salt,
    )
    .await
    .map_err(Self::map_db_error)
  }

  pub async fn create_user_for_admin(
    &self,
    current_user_id: &str,
    input: CreateUserInput,
  ) -> Result<users::Model, UserServiceError> {
    self.require_super_admin_user(current_user_id).await?;

    let username = input.username.trim().to_string();
    let email = input.email.trim().to_ascii_lowercase();
    let password = input.password.trim().to_string();

    if username.is_empty() || email.is_empty() || password.is_empty() {
      return Err(UserServiceError::BadRequest(
        "username/email/password cannot be empty".to_string(),
      ));
    }

    let duplicated = UsersRepository::find_duplicate_user_by_username_or_email(
      &self.db_conn,
      username.as_str(),
      email.as_str(),
    )
    .await
    .map_err(Self::internal_error)?
    .is_some();
    if duplicated {
      return Err(UserServiceError::Conflict(
        "username or email already exists".to_string(),
      ));
    }

    UsersRepository::insert_user(
      &self.db_conn,
      users::ActiveModel::from(CreateUser {
        username,
        email,
        password,
      }),
    )
    .await
    .map_err(Self::map_db_error)
  }

  pub async fn list_users_for_admin(
    &self,
    current_user_id: &str,
    limit: Option<u64>,
  ) -> Result<Vec<users::Model>, UserServiceError> {
    self.require_super_admin_user(current_user_id).await?;

    let resolved_limit = limit.unwrap_or(100);
    if !(1..=200).contains(&resolved_limit) {
      return Err(UserServiceError::BadRequest(
        "limit must be in range [1, 200]".to_string(),
      ));
    }

    UsersRepository::list_active_users(&self.db_conn, Some(resolved_limit))
      .await
      .map_err(Self::internal_error)
  }

  pub async fn list_users_by_ids_for_admin(
    &self,
    current_user_id: &str,
    user_ids: Vec<String>,
  ) -> Result<Vec<users::Model>, UserServiceError> {
    self.require_super_admin_user(current_user_id).await?;

    UsersRepository::list_active_users_by_ids(&self.db_conn, user_ids)
      .await
      .map_err(Self::internal_error)
  }

  pub async fn get_user_by_id_for_admin(
    &self,
    current_user_id: &str,
    user_id: &str,
  ) -> Result<users::Model, UserServiceError> {
    self.require_super_admin_user(current_user_id).await?;

    UsersRepository::find_active_user_by_id(&self.db_conn, user_id)
      .await
      .map_err(Self::internal_error)?
      .ok_or_else(|| UserServiceError::NotFound("user not found".to_string()))
  }

  pub fn is_super_admin(&self, user: &users::Model) -> bool {
    if self.super_admin_identities.is_empty() {
      return false;
    }

    let username = normalize_identity(user.username.as_str());
    let email = normalize_identity(user.email.as_str());
    self.super_admin_identities.contains(username.as_str())
      || self.super_admin_identities.contains(email.as_str())
  }

  fn map_db_error(err: DbErr) -> UserServiceError {
    let message = err.to_string().to_lowercase();
    if message.contains("unique") || message.contains("duplicate") {
      UserServiceError::Conflict(err.to_string())
    } else {
      Self::internal_error(err)
    }
  }

  fn internal_error(err: DbErr) -> UserServiceError {
    UserServiceError::Internal(format!("internal server error: {err}"))
  }

  async fn require_super_admin_user(
    &self,
    current_user_id: &str,
  ) -> Result<users::Model, UserServiceError> {
    let current = self.get_current_user(current_user_id).await?;
    if !self.is_super_admin(&current) {
      return Err(UserServiceError::Forbidden(
        "super admin permission is required".to_string(),
      ));
    }
    Ok(current)
  }
}

fn collect_super_admin_identities(config: &Config) -> HashSet<String> {
  let mut identities = HashSet::new();

  if let Some(auth) = config.auth.as_ref() {
    if let Some(values) = auth.super_admins.as_ref() {
      for value in values {
        let normalized = normalize_identity(value.as_str());
        if !normalized.is_empty() {
          identities.insert(normalized);
        }
      }
    }

    if let Some(admin_username) = auth
      .admin
      .as_ref()
      .and_then(|admin| admin.username.as_ref())
    {
      let normalized = normalize_identity(admin_username.as_str());
      if !normalized.is_empty() {
        identities.insert(normalized);
      }
    }
  }

  identities
}

fn normalize_identity(value: &str) -> String {
  value.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::configuration::cfg::{Admin, Auth};

  #[test]
  fn collect_super_admin_identities_uses_both_list_and_legacy_admin_username() {
    let mut cfg = Config::default();
    cfg.auth = Some(Auth {
      enable_jwt: None,
      jwt_secret: None,
      enable_ldap: None,
      ldap_url: None,
      super_admins: Some(vec![" Root ".to_string(), "admin@example.com".to_string()]),
      admin: Some(Admin {
        username: Some("legacy_admin".to_string()),
        password: None,
      }),
    });

    let identities = collect_super_admin_identities(&cfg);
    assert!(identities.contains("root"));
    assert!(identities.contains("admin@example.com"));
    assert!(identities.contains("legacy_admin"));
  }

  #[test]
  fn normalize_identity_trims_and_lowercases() {
    assert_eq!(
      normalize_identity("  Alice@Example.COM "),
      "alice@example.com"
    );
  }
}
