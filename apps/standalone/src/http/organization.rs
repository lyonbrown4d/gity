use crate::http::app_state::AppState;
use crate::http::auth::ErrorResponse;
use crate::http::pagination::{resolve_pagination, to_page, to_single_page};
use crate::security::current_user::CurrentUser;
use crate::security::organization_acl::{
  RequiredOrganizationRole, member_role_to_string, require_organization_role,
};
use crate::service::organization_service::OrganizationServiceError;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use domain::api::crud::parse_csv_ids;
use domain::api::organization::{
  AddOrganizationMemberRequest, CreateOrganizationRequest, ListOrganizationsQuery,
  OrganizationMemberDetailView, OrganizationMemberView, OrganizationView,
  UpdateOrganizationRequest,
};
use domain::api::response::{ApiResponse, EmptyData};
use domain::page::Page;
use entity::organization_invitations;
use repository::{OrganizationMembersRepository, OrganizationsRepository, UsersRepository};
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

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
  tag = "Organizations",
  params(ListOrganizationsQuery),
  responses(
    (status = 200, description = "Organizations visible to current user", body = ApiResponse<Page<OrganizationView>>),
    (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn list_organizations(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Query(query): Query<ListOrganizationsQuery>,
) -> Result<(StatusCode, Json<Page<OrganizationView>>), (StatusCode, Json<ErrorResponse>)> {
  list_organizations_internal(&state, &current_user, query).await
}

#[utoipa::path(
  get,
  path = "/me",
  tag = "Organizations",
  params(ListOrganizationsQuery),
  responses(
    (status = 200, description = "Organizations of current user", body = ApiResponse<Page<OrganizationView>>),
    (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn list_my_organizations(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Query(query): Query<ListOrganizationsQuery>,
) -> Result<(StatusCode, Json<Page<OrganizationView>>), (StatusCode, Json<ErrorResponse>)> {
  list_organizations_internal(&state, &current_user, query).await
}

async fn list_organizations_internal(
  state: &AppState,
  current_user: &CurrentUser,
  query: ListOrganizationsQuery,
) -> Result<(StatusCode, Json<Page<OrganizationView>>), (StatusCode, Json<ErrorResponse>)> {
  let ids = parse_csv_ids(query.ids.as_deref());
  let has_ids_filter = !ids.is_empty();
  let pagination = if has_ids_filter {
    None
  } else {
    Some(resolve_pagination(query.page, query.page_size, 50, 200)?)
  };

  if current_user_is_super_admin(state, current_user.user_id.as_str()).await? {
    if has_ids_filter {
      let organizations = OrganizationsRepository::new(state.db_conn.clone())
        .list_active_organizations_by_ids(ids.clone())
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
      return Ok((StatusCode::OK, Json(to_single_page(data))));
    }

    let pagination = pagination.ok_or_else(|| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          message: "pagination state is invalid".to_string(),
        }),
      )
    })?;
    let (organizations, total) = OrganizationsRepository::new(state.db_conn.clone())
      .list_active_organizations_paginated(pagination.page, pagination.page_size)
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
    return Ok((StatusCode::OK, Json(to_page(data, total, pagination))));
  }

  let memberships = OrganizationMembersRepository::new(state.db_conn.clone())
    .list_active_memberships_by_user(&current_user.user_id)
    .await
    .map_err(|err| internal_error("failed to load organization memberships", err))?;

  let ids_filter = ids.into_iter().collect::<std::collections::HashSet<_>>();
  let memberships = if has_ids_filter {
    memberships
      .into_iter()
      .filter(|membership| ids_filter.contains(&membership.organization_id))
      .collect()
  } else {
    memberships
  };

  if has_ids_filter {
    if memberships.is_empty() {
      return Ok((StatusCode::OK, Json(to_single_page(Vec::new()))));
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

    let organizations = OrganizationsRepository::new(state.db_conn.clone())
      .list_active_organizations_by_ids(organization_ids)
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

    return Ok((StatusCode::OK, Json(to_single_page(data))));
  }

  let pagination = pagination.ok_or_else(|| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrorResponse {
        message: "pagination state is invalid".to_string(),
      }),
    )
  })?;
  let total = memberships.len() as u64;
  if memberships.is_empty() {
    return Ok((StatusCode::OK, Json(to_page(Vec::new(), total, pagination))));
  }

  let offset = ((pagination.page - 1) * pagination.page_size) as usize;
  let paged_memberships = memberships
    .into_iter()
    .skip(offset)
    .take(pagination.page_size as usize)
    .collect::<Vec<_>>();

  if paged_memberships.is_empty() {
    return Ok((StatusCode::OK, Json(to_page(Vec::new(), total, pagination))));
  }

  let role_by_org: HashMap<String, String> = paged_memberships
    .iter()
    .map(|membership| {
      (
        membership.organization_id.clone(),
        member_role_to_string(membership.role.clone()),
      )
    })
    .collect();
  let organization_ids: Vec<String> = paged_memberships
    .iter()
    .map(|membership| membership.organization_id.clone())
    .collect();

  let organizations = OrganizationsRepository::new(state.db_conn.clone())
    .list_active_organizations_by_ids(organization_ids)
    .await
    .map_err(|err| internal_error("failed to load organizations", err))?;
  let organizations_by_id = organizations
    .into_iter()
    .map(|organization| (organization.id.clone(), organization))
    .collect::<HashMap<_, _>>();

  let data = paged_memberships
    .into_iter()
    .filter_map(|membership| {
      organizations_by_id
        .get(membership.organization_id.as_str())
        .map(|organization| OrganizationView {
          role: role_by_org
            .get(&organization.id)
            .cloned()
            .unwrap_or_else(|| "member".to_string()),
          id: organization.id.clone(),
          key: organization.key.clone(),
          name: organization.name.clone(),
        })
    })
    .collect();

  Ok((StatusCode::OK, Json(to_page(data, total, pagination))))
}

#[utoipa::path(
  get,
  path = "/{organization_id}",
  tag = "Organizations",
  responses(
    (status = 200, description = "Get organization by id", body = ApiResponse<OrganizationView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Organization not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn get_organization(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(organization_id): Path<String>,
) -> Result<(StatusCode, Json<OrganizationView>), (StatusCode, Json<ErrorResponse>)> {
  let role = if current_user_is_super_admin(&state, current_user.user_id.as_str()).await? {
    "super_admin".to_string()
  } else {
    let membership = require_organization_role(
      &state.db_conn,
      current_user.user_id.as_str(),
      organization_id.as_str(),
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
    member_role_to_string(membership.role)
  };

  let organization = OrganizationsRepository::new(state.db_conn.clone())
    .find_active_organization_by_id(organization_id.as_str())
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

  Ok((
    StatusCode::OK,
    Json(OrganizationView {
      id: organization.id,
      key: organization.key,
      name: organization.name,
      role,
    }),
  ))
}

#[utoipa::path(
  post,
  path = "/",
  tag = "Organizations",
  request_body = CreateOrganizationRequest,
  responses(
    (status = 201, description = "Organization created", body = ApiResponse<OrganizationView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Organization key already exists", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Organizations",
  request_body = UpdateOrganizationRequest,
  responses(
    (status = 200, description = "Organization updated", body = ApiResponse<OrganizationView>),
    (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Organization not found", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Organization key already exists", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Organizations",
  responses(
    (status = 200, description = "Organization deleted", body = ApiResponse<EmptyData>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Organization not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn delete_organization(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(organization_id): Path<String>,
) -> Result<(StatusCode, Json<EmptyData>), (StatusCode, Json<ErrorResponse>)> {
  state
    .services
    .organization
    .delete_organization(current_user.user_id.as_str(), organization_id.as_str())
    .await
    .map_err(map_organization_service_error)?;
  Ok((StatusCode::OK, Json(EmptyData {})))
}

#[utoipa::path(
  post,
  path = "/{organization_id}/members",
  tag = "Organizations",
  request_body = AddOrganizationMemberRequest,
  responses(
    (status = 201, description = "Member added to organization", body = ApiResponse<OrganizationMemberView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "User not found", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Member already exists", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Organizations",
  responses(
    (status = 200, description = "Organization members", body = ApiResponse<Vec<OrganizationMemberDetailView>>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Organization not found", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn list_organization_members(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path(organization_id): Path<String>,
) -> Result<(StatusCode, Json<Vec<OrganizationMemberDetailView>>), (StatusCode, Json<ErrorResponse>)>
{
  let organization = OrganizationsRepository::new(state.db_conn.clone())
    .find_active_organization_by_id(organization_id.as_str())
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

  let members = OrganizationMembersRepository::new(state.db_conn.clone())
    .list_active_memberships_by_organization(organization.id.as_str())
    .await
    .map_err(|err| internal_error("failed to load organization members", err))?;
  let user_ids = members
    .iter()
    .map(|member| member.user_id.clone())
    .collect::<Vec<_>>();
  let users = UsersRepository::new(state.db_conn.clone())
    .list_active_users_by_ids(user_ids)
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
  tag = "Organizations",
  request_body = CreateInvitationRequest,
  responses(
    (status = 201, description = "Invitation created", body = ApiResponse<InvitationCreateResponse>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Pending invitation already exists", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Organizations",
  responses(
    (status = 200, description = "Invitation accepted", body = ApiResponse<OrganizationMemberView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Invitation not found", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Invitation already handled", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Organizations",
  request_body = AcceptInvitationByTokenRequest,
  responses(
    (status = 200, description = "Invitation accepted by token", body = ApiResponse<OrganizationMemberView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Invitation not found", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Invitation already handled", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Organizations",
  params(AcceptInvitationByTokenQuery),
  responses(
    (status = 200, description = "Invitation accepted by token query", body = ApiResponse<OrganizationMemberView>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Invitation not found", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Invitation already handled", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
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
  tag = "Organizations",
  responses(
    (status = 200, description = "Invitation revoked", body = ApiResponse<EmptyData>),
    (status = 401, description = "Unauthorized", body = ApiResponse<ErrorResponse>),
    (status = 403, description = "Forbidden", body = ApiResponse<ErrorResponse>),
    (status = 404, description = "Invitation not found", body = ApiResponse<ErrorResponse>),
    (status = 409, description = "Invitation not pending", body = ApiResponse<ErrorResponse>),
    (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>)
  )
)]
pub async fn revoke_organization_invitation(
  State(state): State<AppState>,
  current_user: CurrentUser,
  Path((organization_id, invitation_id)): Path<(String, String)>,
) -> Result<(StatusCode, Json<EmptyData>), (StatusCode, Json<ErrorResponse>)> {
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

  Ok((StatusCode::OK, Json(EmptyData {})))
}

pub fn organization_routes() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes![list_organizations])
    .routes(routes![list_my_organizations])
    .routes(routes![get_organization])
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
    crate::service::user_service::UserServiceError::NotFound(message) => {
      (StatusCode::NOT_FOUND, message)
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
