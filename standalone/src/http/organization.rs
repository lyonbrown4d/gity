use crate::http::app_state::AppState;
use crate::http::auth::ErrorResponse;
use crate::security::current_user::CurrentUser;
use crate::security::organization_acl::{
  RequiredOrganizationRole, member_role_to_string, require_organization_role,
};
use crate::service::organization_service::OrganizationServiceError;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use entity::organization_invitations;
use repository::{OrganizationMembersRepository, OrganizationsRepository, UsersRepository};
use sea_orm::DbErr;
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
pub struct UpdateOrganizationRequest {
  pub key: Option<String>,
  pub name: Option<String>,
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

#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationMemberDetailView {
  pub organization_id: String,
  pub user_id: String,
  pub username: String,
  pub email: String,
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
  path = "/",
  responses(
    (status = 200, description = "Organizations visible to current user", body = [OrganizationView]),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn list_organizations(
  State(state): State<AppState>,
  current_user: CurrentUser,
) -> Result<(StatusCode, Json<Vec<OrganizationView>>), (StatusCode, Json<ErrorResponse>)> {
  list_organizations_internal(&state, &current_user).await
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
  list_organizations_internal(&state, &current_user).await
}

async fn list_organizations_internal(
  state: &AppState,
  current_user: &CurrentUser,
) -> Result<(StatusCode, Json<Vec<OrganizationView>>), (StatusCode, Json<ErrorResponse>)> {
  if current_user_is_super_admin(state, current_user.user_id.as_str()).await? {
    let organizations = OrganizationsRepository::list_active_organizations(&state.db_conn)
      .await
      .map_err(|err| internal_error("failed to load organizations", err))?;

    let data = organizations
      .into_iter()
      .map(|organization| OrganizationView {
        id: organization.id,
        key: organization.key,
        name: organization.name,
        role: "super_admin".to_string(),
      })
      .collect();
    return Ok((StatusCode::OK, Json(data)));
  }

  let memberships = OrganizationMembersRepository::list_active_memberships_by_user(
    &state.db_conn,
    &current_user.user_id,
  )
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

  let organizations =
    OrganizationsRepository::list_active_organizations_by_ids(&state.db_conn, organization_ids)
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
  let created = state
    .services
    .organization
    .create_organization(current_user.user_id.as_str(), payload.key, payload.name)
    .await
    .map_err(map_organization_service_error)?;

  Ok((
    StatusCode::CREATED,
    Json(OrganizationView {
      id: created.id,
      key: created.key,
      name: created.name,
      role: created.role,
    }),
  ))
}

#[utoipa::path(
  patch,
  path = "/{organization_id}",
  request_body = UpdateOrganizationRequest,
  responses(
    (status = 200, description = "Organization updated", body = OrganizationView),
    (status = 400, description = "Bad request", body = ErrorResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Organization not found", body = ErrorResponse),
    (status = 409, description = "Organization key already exists", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn update_organization(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(organization_id): Path<String>,
  Json(payload): Json<UpdateOrganizationRequest>,
) -> Result<(StatusCode, Json<OrganizationView>), (StatusCode, Json<ErrorResponse>)> {
  let updated = state
    .services
    .organization
    .update_organization(
      current_user.user_id.as_str(),
      organization_id.as_str(),
      payload.key,
      payload.name,
    )
    .await
    .map_err(map_organization_service_error)?;

  Ok((
    StatusCode::OK,
    Json(OrganizationView {
      id: updated.id,
      key: updated.key,
      name: updated.name,
      role: updated.role,
    }),
  ))
}

#[utoipa::path(
  delete,
  path = "/{organization_id}",
  responses(
    (status = 204, description = "Organization deleted"),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Organization not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn delete_organization(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(organization_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
  state
    .services
    .organization
    .delete_organization(current_user.user_id.as_str(), organization_id.as_str())
    .await
    .map_err(map_organization_service_error)?;
  Ok(StatusCode::NO_CONTENT)
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
  let member = state
    .services
    .organization
    .add_organization_member(
      current_user.user_id.as_str(),
      organization_id.as_str(),
      payload.user_id,
      payload.role,
    )
    .await
    .map_err(map_organization_service_error)?;

  Ok((
    StatusCode::CREATED,
    Json(OrganizationMemberView {
      organization_id: member.organization_id,
      user_id: member.user_id,
      role: member.role,
    }),
  ))
}

#[utoipa::path(
  get,
  path = "/{organization_id}/members",
  responses(
    (status = 200, description = "Organization members", body = [OrganizationMemberDetailView]),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 403, description = "Forbidden", body = ErrorResponse),
    (status = 404, description = "Organization not found", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  )
)]
pub async fn list_organization_members(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(organization_id): Path<String>,
) -> Result<(StatusCode, Json<Vec<OrganizationMemberDetailView>>), (StatusCode, Json<ErrorResponse>)>
{
  let organization = OrganizationsRepository::find_active_organization_by_id(
    &state.db_conn,
    organization_id.as_str(),
  )
  .await
  .map_err(|err| internal_error("failed to load organization", err))?
  .ok_or_else(|| {
    (
      StatusCode::NOT_FOUND,
      Json(ErrorResponse {
        message: "organization not found".to_string(),
      }),
    )
  })?;

  require_org_member_or_super_admin(
    &state,
    current_user.user_id.as_str(),
    organization.id.as_str(),
  )
  .await?;

  let members = OrganizationMembersRepository::list_active_memberships_by_organization(
    &state.db_conn,
    organization.id.as_str(),
  )
  .await
  .map_err(|err| internal_error("failed to load organization members", err))?;
  let user_ids = members
    .iter()
    .map(|member| member.user_id.clone())
    .collect::<Vec<_>>();
  let users = UsersRepository::list_active_users_by_ids(&state.db_conn, user_ids)
    .await
    .map_err(|err| internal_error("failed to load users", err))?;
  let user_map = users
    .into_iter()
    .map(|user| (user.id.clone(), user))
    .collect::<HashMap<_, _>>();

  let data = members
    .into_iter()
    .filter_map(|member| {
      user_map
        .get(member.user_id.as_str())
        .map(|user| OrganizationMemberDetailView {
          organization_id: member.organization_id,
          user_id: member.user_id,
          username: user.username.clone(),
          email: user.email.clone(),
          role: member_role_to_string(member.role),
        })
    })
    .collect();
  Ok((StatusCode::OK, Json(data)))
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
  let invitation_secret = invitation_secret(&state);
  let public_base_url = public_base_url(&state);
  let invitation = state
    .services
    .organization
    .create_invitation(
      current_user.user_id.as_str(),
      organization_id.as_str(),
      payload.email,
      payload.role,
      payload.expires_in_hours,
      invitation_secret.as_str(),
      public_base_url.as_str(),
    )
    .await
    .map_err(map_organization_service_error)?;

  Ok((
    StatusCode::CREATED,
    Json(InvitationCreateResponse {
      invitation: invitation_view(invitation.invitation),
      invitation_token: invitation.invitation_token,
      acceptance_url: invitation.acceptance_url,
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
  let member = state
    .services
    .organization
    .accept_invitation(current_user.user_id.as_str(), invitation_id, None, None)
    .await
    .map_err(map_organization_service_error)?;

  Ok((
    StatusCode::OK,
    Json(OrganizationMemberView {
      organization_id: member.organization_id,
      user_id: member.user_id,
      role: member.role,
    }),
  ))
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
  let invitation_secret = invitation_secret(&state);
  let member = state
    .services
    .organization
    .accept_invitation_with_token(
      current_user.user_id.as_str(),
      payload.token.as_str(),
      invitation_secret.as_str(),
    )
    .await
    .map_err(map_organization_service_error)?;

  Ok((
    StatusCode::OK,
    Json(OrganizationMemberView {
      organization_id: member.organization_id,
      user_id: member.user_id,
      role: member.role,
    }),
  ))
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
  let invitation_secret = invitation_secret(&state);
  let member = state
    .services
    .organization
    .accept_invitation_with_token(
      current_user.user_id.as_str(),
      query.token.as_str(),
      invitation_secret.as_str(),
    )
    .await
    .map_err(map_organization_service_error)?;

  Ok((
    StatusCode::OK,
    Json(OrganizationMemberView {
      organization_id: member.organization_id,
      user_id: member.user_id,
      role: member.role,
    }),
  ))
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
  state
    .services
    .organization
    .revoke_invitation(
      current_user.user_id.as_str(),
      organization_id.as_str(),
      invitation_id,
    )
    .await
    .map_err(map_organization_service_error)?;

  Ok(StatusCode::NO_CONTENT)
}

pub fn organization_routes() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes![list_organizations])
    .routes(routes![list_my_organizations])
    .routes(routes![create_organization])
    .routes(routes![update_organization])
    .routes(routes![delete_organization])
    .routes(routes![list_organization_members])
    .routes(routes![add_organization_member])
    .routes(routes![create_organization_invitation])
    .routes(routes![accept_organization_invitation])
    .routes(routes![accept_organization_invitation_by_token])
    .routes(routes![accept_organization_invitation_by_token_query])
    .routes(routes![revoke_organization_invitation])
}

fn invitation_secret(state: &AppState) -> String {
  state.services.auth.jwt_secret().to_string()
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

fn internal_error(message: &str, err: DbErr) -> (StatusCode, Json<ErrorResponse>) {
  (
    StatusCode::INTERNAL_SERVER_ERROR,
    Json(ErrorResponse {
      message: format!("{message}: {err}"),
    }),
  )
}

async fn current_user_is_super_admin(
  state: &AppState,
  user_id: &str,
) -> Result<bool, (StatusCode, Json<ErrorResponse>)> {
  let user = state
    .services
    .user
    .get_current_user(user_id)
    .await
    .map_err(map_user_service_error_as_org_error)?;
  Ok(state.services.user.is_super_admin(&user))
}

async fn require_org_member_or_super_admin(
  state: &AppState,
  user_id: &str,
  organization_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
  if current_user_is_super_admin(state, user_id).await? {
    return Ok(());
  }

  require_organization_role(
    &state.db_conn,
    user_id,
    organization_id,
    RequiredOrganizationRole::Member,
  )
  .await
  .map_err(|err| {
    (
      err.status,
      Json(ErrorResponse {
        message: err.message,
      }),
    )
  })?;
  Ok(())
}

fn map_user_service_error_as_org_error(
  err: crate::service::user_service::UserServiceError,
) -> (StatusCode, Json<ErrorResponse>) {
  let (status, message) = match err {
    crate::service::user_service::UserServiceError::BadRequest(message) => {
      (StatusCode::BAD_REQUEST, message)
    }
    crate::service::user_service::UserServiceError::Unauthorized(message) => {
      (StatusCode::UNAUTHORIZED, message)
    }
    crate::service::user_service::UserServiceError::Forbidden(message) => {
      (StatusCode::FORBIDDEN, message)
    }
    crate::service::user_service::UserServiceError::Conflict(message) => {
      (StatusCode::CONFLICT, message)
    }
    crate::service::user_service::UserServiceError::Internal(message) => {
      (StatusCode::INTERNAL_SERVER_ERROR, message)
    }
  };
  (status, Json(ErrorResponse { message }))
}

fn map_organization_service_error(
  err: OrganizationServiceError,
) -> (StatusCode, Json<ErrorResponse>) {
  let (status, message) = match err {
    OrganizationServiceError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
    OrganizationServiceError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
    OrganizationServiceError::Forbidden(message) => (StatusCode::FORBIDDEN, message),
    OrganizationServiceError::NotFound(message) => (StatusCode::NOT_FOUND, message),
    OrganizationServiceError::Conflict(message) => (StatusCode::CONFLICT, message),
    OrganizationServiceError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
  };
  (status, Json(ErrorResponse { message }))
}
