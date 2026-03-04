use crate::configuration::cfg::Config;
use crate::security::jwt::{issue_invitation_token, verify_invitation_token};
use crate::security::organization_acl::{
  RequiredOrganizationRole, member_role_to_string, parse_member_role, require_organization_role,
};
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use entity::{organization_invitations, organization_members, organizations};
use repository::AppRepository;
use sea_orm::{DatabaseConnection, DbErr, Set, TransactionTrait};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct CreatedOrganization {
  pub id: String,
  pub key: String,
  pub name: String,
  pub role: String,
}

#[derive(Debug, Clone)]
pub struct AddedOrganizationMember {
  pub organization_id: String,
  pub user_id: String,
  pub role: String,
}

#[derive(Debug, Clone)]
pub struct CreatedInvitation {
  pub invitation: organization_invitations::Model,
  pub invitation_token: String,
  pub acceptance_url: String,
}

#[derive(Debug)]
pub enum OrganizationServiceError {
  BadRequest(String),
  Unauthorized(String),
  Forbidden(String),
  NotFound(String),
  Conflict(String),
  Internal(String),
}

#[derive(Clone)]
pub struct OrganizationService {
  db_conn: DatabaseConnection,
  super_admin_identities: HashSet<String>,
}

impl OrganizationService {
  pub fn new(config: &Config, db_conn: DatabaseConnection) -> Self {
    Self {
      db_conn,
      super_admin_identities: collect_super_admin_identities(config),
    }
  }

  pub async fn create_organization(
    &self,
    user_id: &str,
    key: String,
    name: String,
  ) -> Result<CreatedOrganization, OrganizationServiceError> {
    let txn = self
      .db_conn
      .begin()
      .await
      .map_err(|err| Self::internal_error("failed to begin transaction", err))?;

    let exists = AppRepository::find_organization_by_key(&txn, key.as_str())
      .await
      .map_err(|err| Self::internal_error("failed to check organization key", err))?
      .is_some();

    if exists {
      return Err(OrganizationServiceError::Conflict(
        "organization key already exists".to_string(),
      ));
    }

    let organization = AppRepository::insert_organization(
      &txn,
      organizations::ActiveModel {
        key: Set(key),
        name: Set(name),
        status: Set(organizations::OrgStatus::Active),
        ..Default::default()
      },
    )
    .await
    .map_err(|err| Self::internal_error("failed to create organization", err))?;

    AppRepository::insert_organization_membership(
      &txn,
      organization_members::ActiveModel {
        organization_id: Set(organization.id.clone()),
        user_id: Set(user_id.to_string()),
        role: Set(organization_members::MemberRole::Owner),
        ..Default::default()
      },
    )
    .await
    .map_err(|err| Self::internal_error("failed to create owner membership", err))?;

    txn
      .commit()
      .await
      .map_err(|err| Self::internal_error("failed to commit transaction", err))?;

    Ok(CreatedOrganization {
      id: organization.id,
      key: organization.key,
      name: organization.name,
      role: "owner".to_string(),
    })
  }

  pub async fn update_organization(
    &self,
    actor_user_id: &str,
    organization_id: &str,
    key: Option<String>,
    name: Option<String>,
  ) -> Result<CreatedOrganization, OrganizationServiceError> {
    self
      .require_owner_or_super_admin(actor_user_id, organization_id)
      .await?;

    let organization =
      AppRepository::find_active_organization_by_id(&self.db_conn, organization_id)
        .await
        .map_err(|err| Self::internal_error("failed to load organization", err))?
        .ok_or_else(|| OrganizationServiceError::NotFound("organization not found".to_string()))?;

    let normalized_key = key.map(|value| value.trim().to_string());
    let normalized_name = name.map(|value| value.trim().to_string());

    if normalized_key
      .as_ref()
      .is_some_and(|value| value.is_empty())
      || normalized_name
        .as_ref()
        .is_some_and(|value| value.is_empty())
    {
      return Err(OrganizationServiceError::BadRequest(
        "organization key/name cannot be empty".to_string(),
      ));
    }

    if normalized_key.is_none() && normalized_name.is_none() {
      return Err(OrganizationServiceError::BadRequest(
        "at least one field must be provided".to_string(),
      ));
    }

    if let Some(new_key) = normalized_key.as_ref() {
      let duplicated = AppRepository::find_active_organization_by_key(&self.db_conn, new_key)
        .await
        .map_err(|err| Self::internal_error("failed to check organization key", err))?;
      if duplicated
        .as_ref()
        .is_some_and(|item| item.id != organization.id)
      {
        return Err(OrganizationServiceError::Conflict(
          "organization key already exists".to_string(),
        ));
      }
    }

    let updated = AppRepository::update_organization(
      &self.db_conn,
      organization,
      normalized_key.clone(),
      normalized_name.clone(),
      None,
      None,
    )
    .await
    .map_err(|err| Self::internal_error("failed to update organization", err))?;

    let role = if self.is_super_admin_user_id(actor_user_id).await? {
      "super_admin".to_string()
    } else {
      "owner".to_string()
    };

    Ok(CreatedOrganization {
      id: updated.id,
      key: updated.key,
      name: updated.name,
      role,
    })
  }

  pub async fn delete_organization(
    &self,
    actor_user_id: &str,
    organization_id: &str,
  ) -> Result<(), OrganizationServiceError> {
    self
      .require_owner_or_super_admin(actor_user_id, organization_id)
      .await?;

    let organization =
      AppRepository::find_active_organization_by_id(&self.db_conn, organization_id)
        .await
        .map_err(|err| Self::internal_error("failed to load organization", err))?
        .ok_or_else(|| OrganizationServiceError::NotFound("organization not found".to_string()))?;

    let now: sea_orm::prelude::DateTimeWithTimeZone = Utc::now().into();
    let txn = self
      .db_conn
      .begin()
      .await
      .map_err(|err| Self::internal_error("failed to begin transaction", err))?;

    let memberships = AppRepository::list_active_memberships_by_organization(
      &txn,
      organization_id,
    )
    .await
    .map_err(|err| Self::internal_error("failed to load organization members", err))?;
    for membership in memberships {
      AppRepository::update_organization_membership(
        &txn,
        membership,
        None,
        Some(Some(now.clone())),
      )
        .await
        .map_err(|err| Self::internal_error("failed to delete organization member", err))?;
    }

    let repositories = AppRepository::list_active_repositories_by_org(&txn, organization_id)
      .await
      .map_err(|err| Self::internal_error("failed to load repositories", err))?;
    for repository in repositories {
      let branches =
        AppRepository::list_repository_branches_by_repo_id(&txn, repository.id.as_str(), false)
          .await
          .map_err(|err| Self::internal_error("failed to load repository branches", err))?;
      for branch in branches {
        AppRepository::update_branch(&txn, branch, None, None, Some(Some(now.clone())))
          .await
          .map_err(|err| Self::internal_error("failed to delete repository branch", err))?;
      }

      AppRepository::update_repository(&txn, repository, None, None, None, Some(Some(now.clone())))
        .await
        .map_err(|err| Self::internal_error("failed to delete repository", err))?;
    }

    AppRepository::update_organization(&txn, organization, None, None, None, Some(Some(now)))
      .await
      .map_err(|err| Self::internal_error("failed to delete organization", err))?;

    txn
      .commit()
      .await
      .map_err(|err| Self::internal_error("failed to commit transaction", err))?;

    Ok(())
  }

  pub async fn add_organization_member(
    &self,
    actor_user_id: &str,
    organization_id: &str,
    target_user_id: String,
    role: Option<String>,
  ) -> Result<AddedOrganizationMember, OrganizationServiceError> {
    self
      .require_owner_or_super_admin(actor_user_id, organization_id)
      .await?;

    let target_user_exists = AppRepository::find_active_user_by_id(&self.db_conn, &target_user_id)
      .await
      .map_err(|err| Self::internal_error("failed to load target user", err))?
      .is_some();

    if !target_user_exists {
      return Err(OrganizationServiceError::NotFound(
        "target user not found".to_string(),
      ));
    }

    let exists =
      AppRepository::exists_active_membership(&self.db_conn, organization_id, &target_user_id)
        .await
        .map_err(|err| Self::internal_error("failed to check existing membership", err))?;

    if exists {
      return Err(OrganizationServiceError::Conflict(
        "user is already a member of this organization".to_string(),
      ));
    }

    let role = parse_member_role(role.as_deref()).ok_or_else(|| {
      OrganizationServiceError::BadRequest("role must be owner or member".to_string())
    })?;
    let role_text = member_role_to_string(role.clone());

    AppRepository::insert_organization_membership(
      &self.db_conn,
      organization_members::ActiveModel {
        organization_id: Set(organization_id.to_string()),
        user_id: Set(target_user_id.clone()),
        role: Set(role),
        ..Default::default()
      },
    )
    .await
    .map_err(|err| Self::internal_error("failed to create organization membership", err))?;

    Ok(AddedOrganizationMember {
      organization_id: organization_id.to_string(),
      user_id: target_user_id,
      role: role_text,
    })
  }

  pub async fn create_invitation(
    &self,
    actor_user_id: &str,
    organization_id: &str,
    email: String,
    role: Option<String>,
    expires_in_hours: Option<i64>,
    invitation_secret: &str,
    public_base_url: &str,
  ) -> Result<CreatedInvitation, OrganizationServiceError> {
    self
      .require_owner_or_super_admin(actor_user_id, organization_id)
      .await?;

    let email = email.trim().to_ascii_lowercase();
    if email.is_empty() {
      return Err(OrganizationServiceError::BadRequest(
        "email is required".to_string(),
      ));
    }

    let existing_pending = AppRepository::find_pending_invitation_by_org_and_email(
      &self.db_conn,
      organization_id,
      email.as_str(),
    )
    .await
    .map_err(|err| Self::internal_error("failed to check pending invitations", err))?;

    if existing_pending.is_some() {
      return Err(OrganizationServiceError::Conflict(
        "pending invitation already exists for this email".to_string(),
      ));
    }

    let role = parse_member_role(role.as_deref()).ok_or_else(|| {
      OrganizationServiceError::BadRequest("role must be owner or member".to_string())
    })?;

    let expires_in_hours = expires_in_hours.unwrap_or(72);
    if !(1..=24 * 30).contains(&expires_in_hours) {
      return Err(OrganizationServiceError::BadRequest(
        "expires_in_hours must be between 1 and 720".to_string(),
      ));
    }

    let invite_role = match role {
      organization_members::MemberRole::Owner => organization_invitations::InvitationRole::Owner,
      organization_members::MemberRole::Member => organization_invitations::InvitationRole::Member,
    };

    let expires_at = Utc::now() + Duration::hours(expires_in_hours);

    let invitation = AppRepository::insert_invitation(
      &self.db_conn,
      organization_invitations::ActiveModel {
        organization_id: Set(organization_id.to_string()),
        email: Set(email.clone()),
        role: Set(invite_role),
        status: Set(organization_invitations::InvitationStatus::Pending),
        invited_by_user_id: Set(actor_user_id.to_string()),
        accepted_by_user_id: Set(None),
        expires_at: Set(Some(expires_at.into())),
        ..Default::default()
      },
    )
    .await
    .map_err(|err| Self::internal_error("failed to create invitation", err))?;

    let invitation_token = issue_invitation_token(
      invitation_secret,
      invitation.id.as_str(),
      invitation.organization_id.as_str(),
      invitation.email.as_str(),
      expires_at,
    )
    .map_err(|err| {
      OrganizationServiceError::Internal(format!("failed to issue invitation token: {err}"))
    })?;

    let acceptance_url = format!(
      "{}/api/v1/orgs/invitations/accept?token={}",
      public_base_url, invitation_token
    );

    Ok(CreatedInvitation {
      invitation,
      invitation_token,
      acceptance_url,
    })
  }

  pub async fn accept_invitation_with_token(
    &self,
    current_user_id: &str,
    token: &str,
    invitation_secret: &str,
  ) -> Result<AddedOrganizationMember, OrganizationServiceError> {
    let claims = verify_invitation_token(invitation_secret, token).map_err(|_| {
      OrganizationServiceError::Unauthorized("invalid or expired invitation token".to_string())
    })?;

    self
      .accept_invitation(
        current_user_id,
        claims.sub,
        Some(claims.email),
        Some(claims.org),
      )
      .await
  }

  pub async fn accept_invitation(
    &self,
    current_user_id: &str,
    invitation_id: String,
    expected_email: Option<String>,
    expected_org: Option<String>,
  ) -> Result<AddedOrganizationMember, OrganizationServiceError> {
    let user = AppRepository::find_active_user_by_id(&self.db_conn, current_user_id)
      .await
      .map_err(|err| Self::internal_error("failed to load current user", err))?
      .ok_or_else(|| {
        OrganizationServiceError::Unauthorized("current user not found".to_string())
      })?;

    let invitation = AppRepository::find_active_invitation_by_id(&self.db_conn, &invitation_id)
      .await
      .map_err(|err| Self::internal_error("failed to load invitation", err))?
      .ok_or_else(|| OrganizationServiceError::NotFound("invitation not found".to_string()))?;

    if invitation.status != organization_invitations::InvitationStatus::Pending {
      return Err(OrganizationServiceError::Conflict(
        "invitation is not pending".to_string(),
      ));
    }

    if let Some(expected_org) = expected_org
      && invitation.organization_id != expected_org
    {
      return Err(OrganizationServiceError::Forbidden(
        "invitation token organization mismatch".to_string(),
      ));
    }

    if let Some(expected_email) = expected_email
      && invitation.email.to_ascii_lowercase() != expected_email.to_ascii_lowercase()
    {
      return Err(OrganizationServiceError::Forbidden(
        "invitation token email mismatch".to_string(),
      ));
    }

    if invitation.email.to_ascii_lowercase() != user.email.to_ascii_lowercase() {
      return Err(OrganizationServiceError::Forbidden(
        "invitation email does not match current user".to_string(),
      ));
    }

    if invitation
      .expires_at
      .is_some_and(|expires_at| expires_at < Utc::now())
    {
      let _ = AppRepository::update_invitation(
        &self.db_conn,
        invitation,
        organization_invitations::InvitationStatus::Expired,
        None,
      )
      .await;
      return Err(OrganizationServiceError::Conflict(
        "invitation has expired".to_string(),
      ));
    }

    let txn = self
      .db_conn
      .begin()
      .await
      .map_err(|err| Self::internal_error("failed to begin transaction", err))?;

    let existing_membership = AppRepository::find_active_membership(
      &txn,
      user.id.as_str(),
      invitation.organization_id.as_str(),
    )
    .await
    .map_err(|err| Self::internal_error("failed to check existing membership", err))?;

    let member_role = match invitation.role {
      organization_invitations::InvitationRole::Owner => organization_members::MemberRole::Owner,
      organization_invitations::InvitationRole::Member => organization_members::MemberRole::Member,
    };

    if existing_membership.is_none() {
      AppRepository::insert_organization_membership(
        &txn,
        organization_members::ActiveModel {
          organization_id: Set(invitation.organization_id.clone()),
          user_id: Set(user.id.clone()),
          role: Set(member_role.clone()),
          ..Default::default()
        },
      )
      .await
      .map_err(|err| Self::internal_error("failed to create organization membership", err))?;
    }

    let organization_id = invitation.organization_id.clone();
    AppRepository::update_invitation(
      &txn,
      invitation,
      organization_invitations::InvitationStatus::Accepted,
      Some(user.id.clone()),
    )
    .await
    .map_err(|err| Self::internal_error("failed to update invitation", err))?;

    txn
      .commit()
      .await
      .map_err(|err| Self::internal_error("failed to commit transaction", err))?;

    Ok(AddedOrganizationMember {
      organization_id,
      user_id: user.id,
      role: member_role_to_string(member_role),
    })
  }

  pub async fn revoke_invitation(
    &self,
    actor_user_id: &str,
    organization_id: &str,
    invitation_id: String,
  ) -> Result<(), OrganizationServiceError> {
    self
      .require_owner_or_super_admin(actor_user_id, organization_id)
      .await?;

    let invitation = AppRepository::find_active_invitation_by_id_and_org(
      &self.db_conn,
      &invitation_id,
      organization_id,
    )
    .await
    .map_err(|err| Self::internal_error("failed to load invitation", err))?
    .ok_or_else(|| OrganizationServiceError::NotFound("invitation not found".to_string()))?;

    if invitation.status != organization_invitations::InvitationStatus::Pending {
      return Err(OrganizationServiceError::Conflict(
        "only pending invitation can be revoked".to_string(),
      ));
    }

    AppRepository::update_invitation(
      &self.db_conn,
      invitation,
      organization_invitations::InvitationStatus::Revoked,
      None,
    )
    .await
    .map_err(|err| Self::internal_error("failed to revoke invitation", err))?;

    Ok(())
  }

  fn map_access_error(
    err: crate::security::organization_acl::AccessError,
  ) -> OrganizationServiceError {
    if err.status == StatusCode::INTERNAL_SERVER_ERROR {
      OrganizationServiceError::Internal(err.message)
    } else {
      OrganizationServiceError::Forbidden(err.message)
    }
  }

  fn internal_error(message: &str, err: DbErr) -> OrganizationServiceError {
    OrganizationServiceError::Internal(format!("{message}: {err}"))
  }

  async fn require_owner_or_super_admin(
    &self,
    actor_user_id: &str,
    organization_id: &str,
  ) -> Result<(), OrganizationServiceError> {
    if self.is_super_admin_user_id(actor_user_id).await? {
      return Ok(());
    }

    require_organization_role(
      &self.db_conn,
      actor_user_id,
      organization_id,
      RequiredOrganizationRole::Owner,
    )
    .await
    .map_err(Self::map_access_error)?;
    Ok(())
  }

  async fn is_super_admin_user_id(&self, user_id: &str) -> Result<bool, OrganizationServiceError> {
    if self.super_admin_identities.is_empty() {
      return Ok(false);
    }

    let user = AppRepository::find_active_user_by_id(&self.db_conn, user_id)
      .await
      .map_err(|err| Self::internal_error("failed to load current user", err))?;
    let Some(user) = user else {
      return Ok(false);
    };

    let username = normalize_identity(user.username.as_str());
    let email = normalize_identity(user.email.as_str());
    Ok(
      self.super_admin_identities.contains(username.as_str())
        || self.super_admin_identities.contains(email.as_str()),
    )
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
