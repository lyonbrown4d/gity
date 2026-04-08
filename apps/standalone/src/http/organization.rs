use crate::http::app_state::AppState;
use crate::http::auth::ErrorResponse;
use crate::http::pagination::{resolve_pagination, to_page, to_single_page};
use crate::security::current_user::CurrentUser;
use crate::security::jwt::{issue_invitation_token, verify_invitation_token};
use crate::service::project_space_service::ProjectSpaceError;
use application::namespace::{
  CreateNamespaceCommand, NamespaceInvitationView, NamespaceView as ProjectSpaceNamespaceView,
  UpdateNamespaceCommand,
};
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
use models::constants::{member_role, namespace_kind, visibility_level};
use repository::UsersRepository;
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
  let ids_filter = ids.into_iter().collect::<std::collections::HashSet<_>>();
  let pagination = if has_ids_filter {
    None
  } else {
    Some(resolve_pagination(query.page, query.page_size, 50, 200)?)
  };

  let is_super_admin = current_user_is_super_admin(state, current_user.user_id.as_str()).await?;
  let mut organizations = state
    .project_space
    .list_group_namespaces_for_user(current_user.user_id.as_str(), is_super_admin)
    .await
    .map_err(map_project_space_error)?;

  if has_ids_filter {
    organizations.retain(|(namespace, _)| ids_filter.contains(namespace.id.to_string().as_str()));
    let data = organizations
      .into_iter()
      .map(|(namespace, role)| organization_view(namespace, role))
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
  let total = organizations.len() as u64;
  if organizations.is_empty() {
    return Ok((StatusCode::OK, Json(to_page(Vec::new(), total, pagination))));
  }

  let offset = ((pagination.page - 1) * pagination.page_size) as usize;
  let data = organizations
    .into_iter()
    .skip(offset)
    .take(pagination.page_size as usize)
    .map(|(namespace, role)| organization_view(namespace, role))
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
  let namespace_id = parse_namespace_id(organization_id.as_str())?;
  let organization = load_group_namespace(&state, namespace_id).await?;
  let role =
    require_namespace_member_role_or_super_admin(&state, &current_user, organization.id).await?;

  Ok((StatusCode::OK, Json(organization_view(organization, role))))
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
    .project_space
    .create_namespace(CreateNamespaceCommand {
      parent_namespace_id: None,
      owner_user_id: Some(current_user.user_id.clone()),
      path_key: payload.key,
      name: payload.name,
      description: None,
      kind: namespace_kind::GROUP.to_string(),
      visibility: visibility_level::PRIVATE.to_string(),
    })
    .await
    .map_err(map_project_space_error)?;

  Ok((
    StatusCode::CREATED,
    Json(organization_view(created, member_role::OWNER.to_string())),
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
  let namespace_id = parse_namespace_id(organization_id.as_str())?;
  let role =
    require_namespace_owner_role_or_super_admin(&state, &current_user, namespace_id).await?;
  let _ = load_group_namespace(&state, namespace_id).await?;

  let updated = state
    .project_space
    .update_namespace(UpdateNamespaceCommand {
      namespace_id,
      path_key: payload.key,
      name: payload.name,
      description: None,
      visibility: None,
    })
    .await
    .map_err(map_project_space_error)?;

  Ok((StatusCode::OK, Json(organization_view(updated, role))))
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
  let namespace_id = parse_namespace_id(organization_id.as_str())?;
  let _ = require_namespace_owner_role_or_super_admin(&state, &current_user, namespace_id).await?;
  let _ = load_group_namespace(&state, namespace_id).await?;

  state
    .project_space
    .delete_namespace(namespace_id)
    .await
    .map_err(map_project_space_error)?;
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
  let namespace_id = parse_namespace_id(organization_id.as_str())?;
  let _ = require_namespace_owner_role_or_super_admin(&state, &current_user, namespace_id).await?;
  let organization = load_group_namespace(&state, namespace_id).await?;

  let user = UsersRepository::new(state.db_conn.clone())
    .find_active_user_by_id(payload.user_id.as_str())
    .await
    .map_err(|err| internal_error("failed to load user", err))?
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          message: "user not found".to_string(),
        }),
      )
    })?;

  let member_role = normalize_organization_member_role(payload.role);
  let member = state
    .project_space
    .add_namespace_member(
      organization.id,
      user.id.as_str(),
      member_role.as_str(),
      None,
    )
    .await
    .map_err(map_project_space_error)?;

  Ok((
    StatusCode::CREATED,
    Json(OrganizationMemberView {
      organization_id: member.namespace_id.to_string(),
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
  let namespace_id = parse_namespace_id(organization_id.as_str())?;
  let organization = load_group_namespace(&state, namespace_id).await?;
  let _ =
    require_namespace_member_role_or_super_admin(&state, &current_user, organization.id).await?;

  let members = state
    .project_space
    .list_namespace_members(organization.id)
    .await
    .map_err(map_project_space_error)?;
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
          organization_id: member.namespace_id.to_string(),
          user_id: member.user_id,
          username: user.username.clone(),
          email: user.email.clone(),
          role: member.role,
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
  let namespace_id = parse_namespace_id(organization_id.as_str())?;
  let _ = require_namespace_owner_role_or_super_admin(&state, &current_user, namespace_id).await?;
  let _ = load_group_namespace(&state, namespace_id).await?;

  let invitation = state
    .project_space
    .create_namespace_invitation(
      namespace_id,
      payload.email.as_str(),
      normalize_organization_member_role(payload.role).as_str(),
      payload.expires_in_hours,
      current_user.user_id.as_str(),
    )
    .await
    .map_err(map_project_space_error)?;

  let invitation_secret = invitation_secret(&state);
  let public_base_url = public_base_url(&state);
  let expires_at = invitation
    .expires_at_unix
    .and_then(|value| chrono::DateTime::from_timestamp(value, 0))
    .ok_or_else(|| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          message: "invitation expiry is missing".to_string(),
        }),
      )
    })?;
  let invitation_token = issue_invitation_token(
    invitation_secret.as_str(),
    invitation.id.to_string().as_str(),
    organization_id.as_str(),
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
    public_base_url, invitation_token
  );

  Ok((
    StatusCode::CREATED,
    Json(InvitationCreateResponse {
      invitation: invitation_view(invitation),
      invitation_token,
      acceptance_url,
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
  let invitation_id = parse_invitation_id(invitation_id.as_str())?;
  let user = current_user_model(&state, current_user.user_id.as_str()).await?;
  let member = state
    .project_space
    .accept_namespace_invitation(
      invitation_id,
      current_user.user_id.as_str(),
      user.email.as_str(),
      None,
      None,
    )
    .await
    .map_err(map_project_space_error)?;

  Ok((
    StatusCode::OK,
    Json(OrganizationMemberView {
      organization_id: member.namespace_id.to_string(),
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
  accept_organization_invitation_by_token_impl(&state, &current_user, payload.token.as_str()).await
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
  accept_organization_invitation_by_token_impl(&state, &current_user, query.token.as_str()).await
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
  let namespace_id = parse_namespace_id(organization_id.as_str())?;
  let invitation_id = parse_invitation_id(invitation_id.as_str())?;
  let _ = require_namespace_owner_role_or_super_admin(&state, &current_user, namespace_id).await?;

  state
    .project_space
    .revoke_namespace_invitation(namespace_id, invitation_id, current_user.user_id.as_str())
    .await
    .map_err(map_project_space_error)?;

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

fn organization_view(namespace: ProjectSpaceNamespaceView, role: String) -> OrganizationView {
  OrganizationView {
    id: namespace.id.to_string(),
    key: namespace.path_key,
    name: namespace.name,
    role,
  }
}

fn parse_namespace_id(organization_id: &str) -> Result<i64, (StatusCode, Json<ErrorResponse>)> {
  organization_id.parse::<i64>().map_err(|_| {
    (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "organization id must be an integer namespace id".to_string(),
      }),
    )
  })
}

async fn load_group_namespace(
  state: &AppState,
  namespace_id: i64,
) -> Result<ProjectSpaceNamespaceView, (StatusCode, Json<ErrorResponse>)> {
  let namespace = state
    .project_space
    .get_namespace(namespace_id)
    .await
    .map_err(map_project_space_error)?
    .ok_or_else(|| {
      (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
          message: "organization not found".to_string(),
        }),
      )
    })?;

  if namespace.kind != namespace_kind::GROUP {
    return Err((
      StatusCode::NOT_FOUND,
      Json(ErrorResponse {
        message: "organization not found".to_string(),
      }),
    ));
  }

  Ok(namespace)
}

async fn require_namespace_member_role_or_super_admin(
  state: &AppState,
  current_user: &CurrentUser,
  namespace_id: i64,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
  require_namespace_role_or_super_admin(
    state,
    current_user,
    namespace_id,
    &[
      member_role::GUEST,
      member_role::REPORTER,
      member_role::DEVELOPER,
      member_role::MAINTAINER,
      member_role::OWNER,
    ],
    "you are not a member of this organization",
  )
  .await
}

async fn require_namespace_owner_role_or_super_admin(
  state: &AppState,
  current_user: &CurrentUser,
  namespace_id: i64,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
  require_namespace_role_or_super_admin(
    state,
    current_user,
    namespace_id,
    &[member_role::OWNER],
    "organization owner permission is required",
  )
  .await
}

async fn require_namespace_role_or_super_admin(
  state: &AppState,
  current_user: &CurrentUser,
  namespace_id: i64,
  accepted_roles: &[&str],
  forbidden_message: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
  if current_user_is_super_admin(state, current_user.user_id.as_str()).await? {
    return Ok("super_admin".to_string());
  }

  let role = state
    .project_space
    .get_namespace_role(namespace_id, current_user.user_id.as_str())
    .await
    .map_err(map_project_space_error)?
    .ok_or_else(|| {
      (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
          message: forbidden_message.to_string(),
        }),
      )
    })?;

  if accepted_roles.iter().any(|item| *item == role) {
    Ok(role)
  } else {
    Err((
      StatusCode::FORBIDDEN,
      Json(ErrorResponse {
        message: forbidden_message.to_string(),
      }),
    ))
  }
}

fn normalize_organization_member_role(role: Option<String>) -> String {
  match role.map(|value| value.trim().to_ascii_lowercase()) {
    None => member_role::DEVELOPER.to_string(),
    Some(value) if value.is_empty() => member_role::DEVELOPER.to_string(),
    Some(value) if value == "member" => member_role::DEVELOPER.to_string(),
    Some(value) => value,
  }
}

async fn accept_organization_invitation_by_token_impl(
  state: &AppState,
  current_user: &CurrentUser,
  token: &str,
) -> Result<(StatusCode, Json<OrganizationMemberView>), (StatusCode, Json<ErrorResponse>)> {
  let invitation_secret = invitation_secret(state);
  let claims = verify_invitation_token(invitation_secret.as_str(), token).map_err(|_| {
    (
      StatusCode::UNAUTHORIZED,
      Json(ErrorResponse {
        message: "invalid or expired invitation token".to_string(),
      }),
    )
  })?;
  let invitation_id = parse_invitation_id(claims.sub.as_str())?;
  let namespace_id = parse_namespace_id(claims.org.as_str())?;
  let user = current_user_model(state, current_user.user_id.as_str()).await?;
  let member = state
    .project_space
    .accept_namespace_invitation(
      invitation_id,
      current_user.user_id.as_str(),
      user.email.as_str(),
      Some(claims.email.as_str()),
      Some(namespace_id),
    )
    .await
    .map_err(map_project_space_error)?;

  Ok((
    StatusCode::OK,
    Json(OrganizationMemberView {
      organization_id: member.namespace_id.to_string(),
      user_id: member.user_id,
      role: member.role,
    }),
  ))
}

async fn current_user_model(
  state: &AppState,
  user_id: &str,
) -> Result<entity::users::Model, (StatusCode, Json<ErrorResponse>)> {
  UsersRepository::new(state.db_conn.clone())
    .find_active_user_by_id(user_id)
    .await
    .map_err(|err| internal_error("failed to load current user", err))?
    .ok_or_else(|| {
      (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
          message: "current user not found".to_string(),
        }),
      )
    })
}

fn parse_invitation_id(invitation_id: &str) -> Result<i64, (StatusCode, Json<ErrorResponse>)> {
  invitation_id.parse::<i64>().map_err(|_| {
    (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "invitation id must be an integer namespace invitation id".to_string(),
      }),
    )
  })
}
fn invitation_secret(state: &AppState) -> String {
  state.services.auth.jwt_secret().to_string()
}

fn public_base_url(state: &AppState) -> String {
  format!("http://localhost:{}", state.config.server.port)
}

fn invitation_view(invitation: NamespaceInvitationView) -> InvitationView {
  InvitationView {
    id: invitation.id.to_string(),
    organization_id: invitation.namespace_id.to_string(),
    email: invitation.email,
    role: invitation.role,
    status: invitation.state,
    expires_at: invitation
      .expires_at_unix
      .and_then(|value| chrono::DateTime::from_timestamp(value, 0))
      .map(|dt| dt.to_rfc3339()),
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

fn map_project_space_error(err: ProjectSpaceError) -> (StatusCode, Json<ErrorResponse>) {
  let (status, message) = match err {
    ProjectSpaceError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
    ProjectSpaceError::NotFound(message) => (StatusCode::NOT_FOUND, message),
    ProjectSpaceError::Conflict(message) => (StatusCode::CONFLICT, message),
    ProjectSpaceError::Forbidden(message) => (StatusCode::FORBIDDEN, message),
    ProjectSpaceError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
  };
  (status, Json(ErrorResponse { message }))
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
