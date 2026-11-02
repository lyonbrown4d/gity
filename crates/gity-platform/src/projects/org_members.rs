use super::org_membership::{Change, mutate};
use super::*;
use crate::openapi::{project_schemas as spec, schemas as api};

use std::collections::HashMap;
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(get,path="/orgs/{id}/members",tag="projects",params(api::ProjectPath),
    responses((status=200,description="Organization members with current user identity and email",body=api::Envelope<Vec<spec::OrganizationMember>>),api::Errors),security(("bearer"=[])))]
pub(super) async fn org_members(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
) -> Reply {
  org_access(&s, &h, id, 0).await?;
  let members = s.projects.organization_members(id).await?;
  let user_ids = members
    .iter()
    .map(|member| member.user_id)
    .collect::<Vec<_>>();
  let users = s
    .projects
    .users(user_ids)
    .await?
    .into_iter()
    .map(|user| (user.id, user))
    .collect::<HashMap<_, _>>();
  let mut rows = Vec::with_capacity(members.len());
  for member in members {
    let user = users
      .get(&member.user_id)
      .ok_or_else(|| AppError::not_found("user not found"))?;
    rows.push(json!({
            "id": member.id,
            "organization_id": member.organization_id,
            "user_id": member.user_id,
            "role": member.role,
            "created_at": member.created_at.and_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            "updated_at": member.updated_at.and_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            "username": user.username,
            "display_name": user.display_name,
            "email": user.email,
        }));
  }
  Ok(ok(rows))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(post,path="/orgs/{id}/members",tag="projects",params(api::ProjectPath),request_body=spec::AddMember,
    responses((status=200,description="Add a new organization member; owner required",body=api::Envelope<spec::OrganizationMember>),api::Errors),security(("bearer"=[])))]
pub(super) async fn add_org_member(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(org): Path<i64>,
  Json(body): Json<Value>,
) -> Reply {
  mutate(
    &s,
    &h,
    org,
    id(&body["user_id"])?,
    Change::Add(super::members::member_role(&body)?),
  )
  .await
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(patch,path="/orgs/{id}/members/{user}",tag="projects",params(spec::MemberPath),request_body=spec::SetMember,
    responses((status=200,description="Update membership; owner required and last owner must remain",body=api::Envelope<spec::OrganizationMember>),api::Errors),security(("bearer"=[])))]
pub(super) async fn set_org_member(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((org, user)): Path<(i64, i64)>,
  Json(body): Json<Value>,
) -> Reply {
  mutate(
    &s,
    &h,
    org,
    user,
    Change::Set(super::members::member_role(&body)?),
  )
  .await
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(delete,path="/orgs/{id}/members/{user}",tag="projects",params(spec::MemberPath),
    responses((status=200,description="Remove membership; owner required and last owner must remain",body=api::Envelope<crate::openapi::user_schemas::OkResult>),api::Errors),security(("bearer"=[])))]
pub(super) async fn delete_org_member(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((org, user)): Path<(i64, i64)>,
) -> Reply {
  mutate(&s, &h, org, user, Change::Remove).await
}
