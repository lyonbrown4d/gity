use crate::configuration::cfg::Config;
use crate::security::jwt::{
  AccessClaims, issue_access_token, issue_refresh_token, verify_access_token, verify_refresh_token,
};
use crate::service::authentication::{
  AuthenticationError, AuthenticationManager, CredentialLogin, DbUserPasswordProvider,
  SuperAdminCredentialsProvider,
};
use domain::user::CreateUser;
use entity::{organization_members, organizations, users};
use repository::{OrganizationMembersRepository, OrganizationsRepository, UsersRepository};
use sea_orm::{DatabaseConnection, DbErr, Set, TransactionTrait};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct RegisterInput {
  pub username: String,
  pub email: String,
  pub password: String,
  pub organization_name: Option<String>,
  pub organization_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoginInput {
  pub username: String,
  pub password: String,
}

#[derive(Debug, Clone)]
pub struct AuthPayload {
  pub user_id: String,
  pub username: String,
  pub organization_id: Option<String>,
  pub organization_name: Option<String>,
  pub token: String,
  pub refresh_token: String,
}

#[derive(Debug)]
pub enum AuthServiceError {
  Conflict(String),
  Unauthorized(String),
  Internal(String),
}

#[derive(Clone)]
pub struct AuthService {
  db_conn: DatabaseConnection,
  jwt_secret: String,
  authentication_manager: AuthenticationManager,
  revoked_tokens: Arc<RwLock<HashMap<String, usize>>>,
}

impl AuthService {
  pub fn new(config: &Config, db_conn: DatabaseConnection) -> Self {
    let jwt_secret = config
      .auth
      .as_ref()
      .and_then(|auth| auth.jwt_secret.clone())
      .filter(|secret| !secret.is_empty())
      .unwrap_or_else(|| "gity-dev-secret-change-me".to_string());
    let authentication_manager = AuthenticationManager::new(vec![
      Arc::new(SuperAdminCredentialsProvider::new(config, db_conn.clone())),
      Arc::new(DbUserPasswordProvider::new(db_conn.clone())),
    ]);

    Self {
      db_conn,
      jwt_secret,
      authentication_manager,
      revoked_tokens: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  pub async fn register(&self, input: RegisterInput) -> Result<AuthPayload, AuthServiceError> {
    let txn = self.db_conn.begin().await.map_err(Self::internal_error)?;

    let duplicated_user =
      UsersRepository::find_duplicate_user_by_username_or_email(&txn, &input.username, &input.email)
        .await
        .map_err(Self::internal_error)?
        .is_some();

    if duplicated_user {
      return Err(AuthServiceError::Conflict(
        "username or email already exists".to_string(),
      ));
    }

    let create_user = CreateUser {
      username: input.username.clone(),
      email: input.email.clone(),
      password: input.password.clone(),
    };

    let user = UsersRepository::insert_user(&txn, users::ActiveModel::from(create_user))
      .await
      .map_err(|err| Self::map_db_error(err, "failed to create user"))?;

    let org_name = input
      .organization_name
      .unwrap_or_else(|| format!("{}'s organization", input.username));
    let org_key = input
      .organization_key
      .unwrap_or_else(|| default_org_key(input.username.as_str()));

    let organization = OrganizationsRepository::insert_organization(
      &txn,
      organizations::ActiveModel {
        key: Set(org_key),
        name: Set(org_name),
        status: Set(organizations::OrgStatus::Active),
        ..Default::default()
      },
    )
    .await
    .map_err(|err| Self::map_db_error(err, "failed to create organization"))?;

    OrganizationMembersRepository::insert_organization_membership(
      &txn,
      organization_members::ActiveModel {
        organization_id: Set(organization.id.clone()),
        user_id: Set(user.id.clone()),
        role: Set(organization_members::MemberRole::Owner),
        ..Default::default()
      },
    )
    .await
    .map_err(|err| Self::map_db_error(err, "failed to create organization membership"))?;

    txn.commit().await.map_err(Self::internal_error)?;

    let token = self
      .issue_access_token(user.id.as_str(), Some(organization.id.as_str()))
      .map_err(|err| AuthServiceError::Internal(format!("failed to issue token: {err}")))?;
    let refresh_token = self
      .issue_refresh_token(user.id.as_str(), Some(organization.id.as_str()))
      .map_err(|err| AuthServiceError::Internal(format!("failed to issue refresh token: {err}")))?;

    Ok(AuthPayload {
      user_id: user.id,
      username: user.username,
      organization_id: Some(organization.id),
      organization_name: Some(organization.name),
      token,
      refresh_token,
    })
  }

  pub async fn login(&self, input: LoginInput) -> Result<AuthPayload, AuthServiceError> {
    let credentials = CredentialLogin {
      username: input.username,
      password: input.password,
    };
    let user = self
      .authentication_manager
      .authenticate(&credentials)
      .await
      .map_err(Self::authentication_error)?
      .ok_or_else(|| {
        AuthServiceError::Unauthorized("invalid username/email or password".to_string())
      })?;

    let membership = OrganizationMembersRepository::find_first_active_membership_by_user(&self.db_conn, &user.id)
      .await
      .map_err(Self::internal_error)?;

    let organization = match membership.as_ref() {
      Some(member) => OrganizationsRepository::find_active_organization_by_id(
        &self.db_conn,
        member.organization_id.as_str(),
      )
      .await
      .map_err(Self::internal_error)?,
      None => None,
    };

    let token = self
      .issue_access_token(
        user.id.as_str(),
        organization.as_ref().map(|org| org.id.as_str()),
      )
      .map_err(|err| AuthServiceError::Internal(format!("failed to issue token: {err}")))?;
    let refresh_token = self
      .issue_refresh_token(
        user.id.as_str(),
        organization.as_ref().map(|org| org.id.as_str()),
      )
      .map_err(|err| AuthServiceError::Internal(format!("failed to issue refresh token: {err}")))?;

    Ok(AuthPayload {
      user_id: user.id,
      username: user.username,
      organization_id: organization.as_ref().map(|org| org.id.clone()),
      organization_name: organization.as_ref().map(|org| org.name.clone()),
      token,
      refresh_token,
    })
  }

  pub async fn refresh(&self, refresh_token: &str) -> Result<AuthPayload, AuthServiceError> {
    if self.is_token_revoked(refresh_token).await {
      return Err(AuthServiceError::Unauthorized(
        "refresh token has been revoked".to_string(),
      ));
    }

    let claims = verify_refresh_token(self.jwt_secret.as_str(), refresh_token).map_err(|_| {
      AuthServiceError::Unauthorized("invalid or expired refresh token".to_string())
    })?;

    let user = UsersRepository::find_active_user_by_id(&self.db_conn, claims.sub.as_str())
      .await
      .map_err(Self::internal_error)?
      .ok_or_else(|| AuthServiceError::Unauthorized("user not found".to_string()))?;

    let organization = match claims.org.as_deref() {
      Some(org_id) => OrganizationsRepository::find_active_organization_by_id(&self.db_conn, org_id)
        .await
        .map_err(Self::internal_error)?,
      None => None,
    };

    self.revoke_token(refresh_token).await?;

    let new_access = self
      .issue_access_token(
        user.id.as_str(),
        organization.as_ref().map(|org| org.id.as_str()),
      )
      .map_err(|err| AuthServiceError::Internal(format!("failed to issue token: {err}")))?;
    let new_refresh = self
      .issue_refresh_token(
        user.id.as_str(),
        organization.as_ref().map(|org| org.id.as_str()),
      )
      .map_err(|err| AuthServiceError::Internal(format!("failed to issue refresh token: {err}")))?;

    Ok(AuthPayload {
      user_id: user.id,
      username: user.username,
      organization_id: organization.as_ref().map(|org| org.id.clone()),
      organization_name: organization.as_ref().map(|org| org.name.clone()),
      token: new_access,
      refresh_token: new_refresh,
    })
  }

  pub async fn logout(
    &self,
    current_user_id: &str,
    access_token: &str,
    refresh_token: Option<&str>,
  ) -> Result<(), AuthServiceError> {
    let access_claims = verify_access_token(self.jwt_secret.as_str(), access_token)
      .map_err(|_| AuthServiceError::Unauthorized("invalid or expired token".to_string()))?;
    if access_claims.sub != current_user_id {
      return Err(AuthServiceError::Unauthorized(
        "token user mismatch".to_string(),
      ));
    }

    self.revoke_token(access_token).await?;

    if let Some(refresh_token) = refresh_token {
      let refresh_claims =
        verify_refresh_token(self.jwt_secret.as_str(), refresh_token).map_err(|_| {
          AuthServiceError::Unauthorized("invalid or expired refresh token".to_string())
        })?;
      if refresh_claims.sub != current_user_id {
        return Err(AuthServiceError::Unauthorized(
          "refresh token user mismatch".to_string(),
        ));
      }
      self.revoke_token(refresh_token).await?;
    }

    Ok(())
  }

  pub fn issue_access_token(
    &self,
    user_id: &str,
    organization_id: Option<&str>,
  ) -> Result<String, String> {
    issue_access_token(self.jwt_secret.as_str(), user_id, organization_id)
      .map_err(|err| err.to_string())
  }

  pub fn verify_access_token(&self, token: &str) -> Result<AccessClaims, String> {
    verify_access_token(self.jwt_secret.as_str(), token).map_err(|err| err.to_string())
  }

  pub async fn verify_access_token_for_request(&self, token: &str) -> Result<AccessClaims, String> {
    if self.is_token_revoked(token).await {
      return Err("token has been revoked".to_string());
    }
    self.verify_access_token(token)
  }

  pub fn issue_refresh_token(
    &self,
    user_id: &str,
    organization_id: Option<&str>,
  ) -> Result<String, String> {
    issue_refresh_token(self.jwt_secret.as_str(), user_id, organization_id)
      .map_err(|err| err.to_string())
  }

  pub fn jwt_secret(&self) -> &str {
    self.jwt_secret.as_str()
  }

  async fn revoke_token(&self, token: &str) -> Result<(), AuthServiceError> {
    let exp = match verify_access_token(self.jwt_secret.as_str(), token) {
      Ok(claims) => claims.exp,
      Err(_) => verify_refresh_token(self.jwt_secret.as_str(), token)
        .map(|claims| claims.exp)
        .map_err(|_| AuthServiceError::Unauthorized("invalid token".to_string()))?,
    };

    let now = chrono::Utc::now().timestamp() as usize;
    if exp <= now {
      return Ok(());
    }

    let mut revoked = self.revoked_tokens.write().await;
    revoked.insert(token.to_string(), exp);
    Ok(())
  }

  async fn is_token_revoked(&self, token: &str) -> bool {
    let now = chrono::Utc::now().timestamp() as usize;
    {
      let revoked = self.revoked_tokens.read().await;
      if let Some(exp) = revoked.get(token) {
        return *exp > now;
      }
    }

    let mut revoked = self.revoked_tokens.write().await;
    revoked.retain(|_, exp| *exp > now);
    revoked.contains_key(token)
  }

  fn map_db_error(err: DbErr, message: &str) -> AuthServiceError {
    if is_unique_violation(&err) {
      AuthServiceError::Conflict(err.to_string())
    } else {
      AuthServiceError::Internal(format!("{message}: {err}"))
    }
  }

  fn internal_error(err: DbErr) -> AuthServiceError {
    AuthServiceError::Internal(format!("internal server error: {err}"))
  }

  fn authentication_error(err: AuthenticationError) -> AuthServiceError {
    match err {
      AuthenticationError::Internal(message) => AuthServiceError::Internal(message),
    }
  }
}

fn default_org_key(username: &str) -> String {
  let normalized = username.trim().to_lowercase();
  let mapped: String = normalized
    .chars()
    .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
    .collect();

  format!("{}-org", collapse_hyphens(mapped))
}

fn collapse_hyphens(input: String) -> String {
  input
    .split('-')
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join("-")
}

fn is_unique_violation(err: &DbErr) -> bool {
  let message = err.to_string().to_lowercase();
  message.contains("unique") || message.contains("duplicate")
}
