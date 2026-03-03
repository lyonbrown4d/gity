use crate::http::app_state::AppState;
use crate::http::auth::ErrorResponse;
use crate::security::current_user::CurrentUser;
use crate::security::jwt::{issue_invitation_token, verify_invitation_token};
use crate::security::organization_acl::{
  AccessError, RequiredOrganizationRole, member_role_to_string, parse_member_role,
  require_organization_role,
};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{Duration, Utc};
use entity::{organization_invitations, organization_members, organizations, users};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Condition, DbErr, EntityTrait, IntoActiveModel, QueryFilter, Set,
  TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationView {
  pub id: String,
  pub key: String,
  pub name: String,
  pub role: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrganizationRequest {
  pub key: String,
  pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddOrganizationMemberRequest {
  pub user_id: String,
  pub role: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationMemberView {
  pub organization_id: String,
  pub user_id: String,
  pub role: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInvitationRequest {
  pub email: String,
  pub role: Option<String>,
  pub expires_in_hours: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AcceptInvitationByTokenRequest {
  pub token: String,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct AcceptInvitationByTokenQuery {
  pub token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvitationView {
  pub id: String,
  pub organization_id: String,
  pub email: String,
  pub role: String,
  pub status: String,
  pub expires_at: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvitationCreateResponse {
  pub invitation: InvitationView,
  pub invitation_token: String,
  pub acceptance_url: String,
}

#[utoipa::path(
  get,
  path = "/me",
  responses(
    (status = 200, description = "Organizations of current user", body = [OrganizationView]),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn list_my_organizations(
  State(state): State<AppState>,
  current_user: CurrentUser,
) -> Result<(StatusCode, Json<Vec<OrganizationView>>), (StatusCode, Json<ErrorResponse>)> {
  let memberships = organization_members::Entity::find()
    .filter(
      Condition::all()
        .add(organization_members::Column::UserId.eq(current_user.user_id))
        .add(organization_members::Column::DeletedAt.is_null()),
    )
    .all(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to load organization memberships", err))?;

  if memberships.is_empty() {
    return Ok((StatusCode::OK, Json(vec![])));
  }

  let organization_ids: Vec<String> = memberships
    .iter()
    .map(|membership| membership.organization_id.clone())
    .collect();

  let role_by_org: HashMap<String, String> = memberships
    .into_iter()
    .map(|membership| {
      (
        membership.organization_id.clone(),
        member_role_to_string(membership.role),
      )
    })
    .collect();

  let organizations = organizations::Entity::find()
    .filter(
      Condition::all()
        .add(organizations::Column::Id.is_in(organization_ids))
        .add(organizations::Column::DeletedAt.is_null()),
    )
    .all(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to load organizations", err))?;

  let data = organizations
    .into_iter()
    .map(|organization| OrganizationView {
      role: role_by_org
        .get(&organization.id)
        .cloned()
        .unwrap_or_else(|| "member".to_string()),
      id: organization.id,
      key: organization.key,
      name: organization.name,
    })
    .collect();

  Ok((StatusCode::OK, Json(data)))
}

#[utoipa::path(
  post,
  path = "/",
  request_body = CreateOrganizationRequest,
  responses(
    (status = 201, description = "Organization created", body = OrganizationView),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 409, description = "Organization key already exists", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn create_organization(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Json(payload): Json<CreateOrganizationRequest>,
) -> Result<(StatusCode, Json<OrganizationView>), (StatusCode, Json<ErrorResponse>)> {
  let txn = state
    .db_conn
    .begin()
    .await
    .map_err(|err| internal_error("failed to begin transaction", err))?;

  let exists = organizations::Entity::find()
    .filter(organizations::Column::Key.eq(payload.key.clone()))
    .one(&txn)
    .await
    .map_err(|err| internal_error("failed to check organization key", err))?
    .is_some();

  if exists {
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorResponse {
        message: "organization key already exists".to_string(),
      }),
    ));
  }

  let organization = organizations::ActiveModel {
    key: Set(payload.key),
    name: Set(payload.name),
    status: Set(organizations::OrgStatus::Active),
    ..Default::default()
  }
  .insert(&txn)
  .await
  .map_err(|err| internal_error("failed to create organization", err))?;

  organization_members::ActiveModel {
    organization_id: Set(organization.id.clone()),
    user_id: Set(current_user.user_id),
    role: Set(organization_members::MemberRole::Owner),
    ..Default::default()
  }
  .insert(&txn)
  .await
  .map_err(|err| internal_error("failed to create owner membership", err))?;

  txn.commit()
    .await
    .map_err(|err| internal_error("failed to commit transaction", err))?;

  Ok((
    StatusCode::CREATED,
    Json(OrganizationView {
      id: organization.id,
      key: organization.key,
      name: organization.name,
      role: "owner".to_string(),
    }),
  ))
}

#[utoipa::path(
  post,
  path = "/{organization_id}/members",
  request_body = AddOrganizationMemberRequest,
  responses(
    (status = 201, description = "Member added to organization", body = OrganizationMemberView),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "User not found", body = ErrorResponse),
    (status = 409, description = "Member already exists", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn add_organization_member(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(organization_id): Path<String>,
  Json(payload): Json<AddOrganizationMemberRequest>,
) -> Result<(StatusCode, Json<OrganizationMemberView>), (StatusCode, Json<ErrorResponse>)> {
  require_organization_role(
    &state.db_conn,
    current_user.user_id.as_str(),
    organization_id.as_str(),
    RequiredOrganizationRole::Owner,
  )
  .await
  .map_err(access_error)?;

  let target_user_exists = users::Entity::find_by_id(payload.user_id.clone())
    .filter(users::Column::DeletedAt.is_null())
    .one(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to load target user", err))?
    .is_some();

  if !target_user_exists {
    return Err((
      StatusCode::NOT_FOUND,
      Json(ErrorResponse {
        message: "target user not found".to_string(),
      }),
    ));
  }

  let exists = organization_members::Entity::find()
    .filter(
      Condition::all()
        .add(organization_members::Column::OrganizationId.eq(organization_id.clone()))
        .add(organization_members::Column::UserId.eq(payload.user_id.clone()))
        .add(organization_members::Column::DeletedAt.is_null()),
    )
    .one(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to check existing membership", err))?
    .is_some();

  if exists {
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorResponse {
        message: "user is already a member of this organization".to_string(),
      }),
    ));
  }

  let role = parse_member_role(payload.role.as_deref()).ok_or_else(|| {
    (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "role must be owner or member".to_string(),
      }),
    )
  })?;
  let role_text = member_role_to_string(role.clone());
  organization_members::ActiveModel {
    organization_id: Set(organization_id.clone()),
    user_id: Set(payload.user_id.clone()),
    role: Set(role),
    ..Default::default()
  }
  .insert(&state.db_conn)
  .await
  .map_err(|err| internal_error("failed to create organization membership", err))?;

  Ok((
    StatusCode::CREATED,
    Json(OrganizationMemberView {
      organization_id,
      user_id: payload.user_id,
      role: role_text,
    }),
  ))
}

#[utoipa::path(
  post,
  path = "/{organization_id}/invitations",
  request_body = CreateInvitationRequest,
  responses(
    (status = 201, description = "Invitation created", body = InvitationCreateResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 409, description = "Pending invitation already exists", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn create_organization_invitation(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(organization_id): Path<String>,
  Json(payload): Json<CreateInvitationRequest>,
) -> Result<(StatusCode, Json<InvitationCreateResponse>), (StatusCode, Json<ErrorResponse>)> {
  require_organization_role(
    &state.db_conn,
    current_user.user_id.as_str(),
    organization_id.as_str(),
    RequiredOrganizationRole::Owner,
  )
  .await
  .map_err(access_error)?;

  let email = payload.email.trim().to_ascii_lowercase();
  if email.is_empty() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "email is required".to_string(),
      }),
    ));
  }

  let existing_pending = organization_invitations::Entity::find()
    .filter(
      Condition::all()
        .add(organization_invitations::Column::OrganizationId.eq(organization_id.clone()))
        .add(organization_invitations::Column::Email.eq(email.clone()))
        .add(
          organization_invitations::Column::Status
            .eq(organization_invitations::InvitationStatus::Pending),
        )
        .add(organization_invitations::Column::DeletedAt.is_null()),
    )
    .one(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to check pending invitations", err))?;

  if existing_pending.is_some() {
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorResponse {
        message: "pending invitation already exists for this email".to_string(),
      }),
    ));
  }

  let role = parse_member_role(payload.role.as_deref()).ok_or_else(|| {
    (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "role must be owner or member".to_string(),
      }),
    )
  })?;
  let expires_in_hours = payload.expires_in_hours.unwrap_or(72);
  if !(1..=24 * 30).contains(&expires_in_hours) {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "expires_in_hours must be between 1 and 720".to_string(),
      }),
    ));
  }

  let invite_role = match role {
    organization_members::MemberRole::Owner => organization_invitations::InvitationRole::Owner,
    organization_members::MemberRole::Member => organization_invitations::InvitationRole::Member,
  };

  let expires_at = Utc::now() + Duration::hours(expires_in_hours);

  let invitation = organization_invitations::ActiveModel {
    organization_id: Set(organization_id.clone()),
    email: Set(email.clone()),
    role: Set(invite_role),
    status: Set(organization_invitations::InvitationStatus::Pending),
    invited_by_user_id: Set(current_user.user_id),
    accepted_by_user_id: Set(None),
    expires_at: Set(Some(expires_at.into())),
    ..Default::default()
  }
  .insert(&state.db_conn)
  .await
  .map_err(|err| internal_error("failed to create invitation", err))?;

  let token = issue_invitation_token(
    invitation_secret(&state).as_str(),
    invitation.id.as_str(),
    invitation.organization_id.as_str(),
    invitation.email.as_str(),
    expires_at,
  )
  .map_err(|err| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse {
        message: format!("failed to issue invitation token: {err}"),
      }),
    )
  })?;

  let acceptance_url = format!(
    "{}/api/v1/orgs/invitations/accept?token={}",
    public_base_url(&state),
    token
  );

  Ok((
    StatusCode::CREATED,
    Json(InvitationCreateResponse {
      invitation: invitation_view(invitation),
      invitation_token: token,
      acceptance_url,
    }),
  ))
}

#[utoipa::path(
  post,
  path = "/invitations/{invitation_id}/accept",
  responses(
    (status = 200, description = "Invitation accepted", body = OrganizationMemberView),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Invitation not found", body = ErrorResponse),
    (status = 409, description = "Invitation already handled", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn accept_organization_invitation(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(invitation_id): Path<String>,
) -> Result<(StatusCode, Json<OrganizationMemberView>), (StatusCode, Json<ErrorResponse>)> {
  accept_invitation_inner(&state, &current_user, invitation_id, None, None).await
}

#[utoipa::path(
  post,
  path = "/invitations/accept",
  request_body = AcceptInvitationByTokenRequest,
  responses(
    (status = 200, description = "Invitation accepted by token", body = OrganizationMemberView),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Invitation not found", body = ErrorResponse),
    (status = 409, description = "Invitation already handled", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn accept_organization_invitation_by_token(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Json(payload): Json<AcceptInvitationByTokenRequest>,
) -> Result<(StatusCode, Json<OrganizationMemberView>), (StatusCode, Json<ErrorResponse>)> {
  accept_invitation_with_token(&state, &current_user, payload.token.as_str()).await
}

#[utoipa::path(
  get,
  path = "/invitations/accept",
  params(AcceptInvitationByTokenQuery),
  responses(
    (status = 200, description = "Invitation accepted by token query", body = OrganizationMemberView),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Invitation not found", body = ErrorResponse),
    (status = 409, description = "Invitation already handled", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn accept_organization_invitation_by_token_query(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Query(query): Query<AcceptInvitationByTokenQuery>,
) -> Result<(StatusCode, Json<OrganizationMemberView>), (StatusCode, Json<ErrorResponse>)> {
  accept_invitation_with_token(&state, &current_user, query.token.as_str()).await
}

#[utoipa::path(
  delete,
  path = "/{organization_id}/invitations/{invitation_id}",
  responses(
    (status = 204, description = "Invitation revoked"),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Invitation not found", body = ErrorResponse),
    (status = 409, description = "Invitation not pending", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn revoke_organization_invitation(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path((organization_id, invitation_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
  require_organization_role(
    &state.db_conn,
    current_user.user_id.as_str(),
    organization_id.as_str(),
    RequiredOrganizationRole::Owner,
  )
  .await
  .map_err(access_error)?;

  let invitation = organization_invitations::Entity::find_by_id(invitation_id)
    .filter(
      Condition::all()
        .add(organization_invitations::Column::OrganizationId.eq(organization_id))
        .add(organization_invitations::Column::DeletedAt.is_null()),
    )
    .one(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to load invitation", err))?
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          message: "invitation not found".to_string(),
        }),
      )
    })?;

  if invitation.status != organization_invitations::InvitationStatus::Pending {
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorResponse {
        message: "only pending invitation can be revoked".to_string(),
      }),
    ));
  }

  let mut active = invitation.into_active_model();
  active.status = Set(organization_invitations::InvitationStatus::Revoked);
  active.updated_at = Set(Utc::now().into());
  active
    .update(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to revoke invitation", err))?;

  Ok(StatusCode::NO_CONTENT)
}

pub fn organization_routes() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes![list_my_organizations])
    .routes(routes![create_organization])
    .routes(routes![add_organization_member])
    .routes(routes![create_organization_invitation])
    .routes(routes![accept_organization_invitation])
    .routes(routes![accept_organization_invitation_by_token])
    .routes(routes![accept_organization_invitation_by_token_query])
    .routes(routes![revoke_organization_invitation])
}

async fn accept_invitation_with_token(
  state: &AppState,
  current_user: &CurrentUser,
  token: &str,
) -> Result<(StatusCode, Json<OrganizationMemberView>), (StatusCode, Json<ErrorResponse>)> {
  let claims = verify_invitation_token(invitation_secret(state).as_str(), token).map_err(|_| {
    (
      StatusCode::UNAUTHORIZED,
      Json(ErrorResponse {
        message: "invalid or expired invitation token".to_string(),
      }),
    )
  })?;

  accept_invitation_inner(
    state,
    current_user,
    claims.sub,
    Some(claims.email),
    Some(claims.org),
  )
  .await
}

async fn accept_invitation_inner(
  state: &AppState,
  current_user: &CurrentUser,
  invitation_id: String,
  expected_email: Option<String>,
  expected_org: Option<String>,
) -> Result<(StatusCode, Json<OrganizationMemberView>), (StatusCode, Json<ErrorResponse>)> {
  let user = users::Entity::find_by_id(current_user.user_id.clone())
    .filter(users::Column::DeletedAt.is_null())
    .one(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to load current user", err))?
    .ok_or_else(|| {
      (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
          message: "current user not found".to_string(),
        }),
      )
    })?;

  let invitation = organization_invitations::Entity::find_by_id(invitation_id.clone())
    .filter(organization_invitations::Column::DeletedAt.is_null())
    .one(&state.db_conn)
    .await
    .map_err(|err| internal_error("failed to load invitation", err))?
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          message: "invitation not found".to_string(),
        }),
      )
    })?;

  if invitation.status != organization_invitations::InvitationStatus::Pending {
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorResponse {
        message: "invitation is not pending".to_string(),
      }),
    ));
  }

  if let Some(expected_org) = expected_org
    && invitation.organization_id != expected_org
  {
    return Err((
      StatusCode::FORBIDDEN,
      Json(ErrorResponse {
        message: "invitation token organization mismatch".to_string(),
      }),
    ));
  }

  if let Some(expected_email) = expected_email
    && invitation.email.to_ascii_lowercase() != expected_email.to_ascii_lowercase()
  {
    return Err((
      StatusCode::FORBIDDEN,
      Json(ErrorResponse {
        message: "invitation token email mismatch".to_string(),
      }),
    ));
  }

  if invitation.email.to_ascii_lowercase() != user.email.to_ascii_lowercase() {
    return Err((
      StatusCode::FORBIDDEN,
      Json(ErrorResponse {
        message: "invitation email does not match current user".to_string(),
      }),
    ));
  }

  if invitation.expires_at.is_some_and(|expires_at| expires_at < Utc::now()) {
    let mut active = invitation.into_active_model();
    active.status = Set(organization_invitations::InvitationStatus::Expired);
    active.updated_at = Set(Utc::now().into());
    let _ = active.update(&state.db_conn).await;
    return Err((
      StatusCode::CONFLICT,
      Json(ErrorResponse {
        message: "invitation has expired".to_string(),
      }),
    ));
  }

  let txn = state
    .db_conn
    .begin()
    .await
    .map_err(|err| internal_error("failed to begin transaction", err))?;

  let existing_membership = organization_members::Entity::find()
    .filter(
      Condition::all()
        .add(organization_members::Column::OrganizationId.eq(invitation.organization_id.clone()))
        .add(organization_members::Column::UserId.eq(user.id.clone()))
        .add(organization_members::Column::DeletedAt.is_null()),
    )
    .one(&txn)
    .await
    .map_err(|err| internal_error("failed to check existing membership", err))?;

  let member_role = match invitation.role {
    organization_invitations::InvitationRole::Owner => organization_members::MemberRole::Owner,
    organization_invitations::InvitationRole::Member => organization_members::MemberRole::Member,
  };

  if existing_membership.is_none() {
    organization_members::ActiveModel {
      organization_id: Set(invitation.organization_id.clone()),
      user_id: Set(user.id.clone()),
      role: Set(member_role.clone()),
      ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(|err| internal_error("failed to create organization membership", err))?;
  }

  let organization_id = invitation.organization_id.clone();
  let mut invitation_active = invitation.into_active_model();
  invitation_active.status = Set(organization_invitations::InvitationStatus::Accepted);
  invitation_active.accepted_by_user_id = Set(Some(user.id.clone()));
  invitation_active.updated_at = Set(Utc::now().into());
  invitation_active
    .update(&txn)
    .await
    .map_err(|err| internal_error("failed to update invitation", err))?;

  txn.commit()
    .await
    .map_err(|err| internal_error("failed to commit transaction", err))?;

  Ok((
    StatusCode::OK,
    Json(OrganizationMemberView {
      organization_id,
      user_id: user.id,
      role: member_role_to_string(member_role),
    }),
  ))
}

fn invitation_secret(state: &AppState) -> String {
  state
    .config
    .auth
    .as_ref()
    .and_then(|auth| auth.jwt_secret.clone())
    .filter(|secret| !secret.is_empty())
    .unwrap_or_else(|| "gity-dev-secret-change-me".to_string())
}

fn public_base_url(state: &AppState) -> String {
  format!("http://localhost:{}", state.config.server.port)
}

fn invitation_view(invitation: organization_invitations::Model) -> InvitationView {
  InvitationView {
    id: invitation.id,
    organization_id: invitation.organization_id,
    email: invitation.email,
    role: match invitation.role {
      organization_invitations::InvitationRole::Owner => "owner".to_string(),
      organization_invitations::InvitationRole::Member => "member".to_string(),
    },
    status: match invitation.status {
      organization_invitations::InvitationStatus::Pending => "pending".to_string(),
      organization_invitations::InvitationStatus::Accepted => "accepted".to_string(),
      organization_invitations::InvitationStatus::Revoked => "revoked".to_string(),
      organization_invitations::InvitationStatus::Expired => "expired".to_string(),
    },
    expires_at: invitation.expires_at.map(|dt| dt.to_rfc3339()),
  }
}

fn access_error(err: AccessError) -> (StatusCode, Json<ErrorResponse>) {
  (
    err.status,
    Json(ErrorResponse {
      message: err.message,
    }),
  )
}

fn internal_error(message: &str, err: DbErr) -> (StatusCode, Json<ErrorResponse>) {
  (
    StatusCode::INTERNAL_SERVER_ERROR,
    Json(ErrorResponse {
      message: format!("{message}: {err}"),
    }),
  )
}
