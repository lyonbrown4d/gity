use super::org_membership::{Change, mutate};
use super::*;
use crate::openapi::{project_schemas as spec, schemas as api};

#[utoipa::path(get,path="/orgs/{id}/members",tag="projects",params(api::ProjectPath),
    responses((status=200,description="Organization members with current user identity and email",body=api::Envelope<Vec<spec::OrganizationMember>>),api::Errors),security(("bearer"=[])))]
pub(super) async fn org_members(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<i64>,
) -> Reply {
    org_access(&s, &h, id, 0).await?;
    Ok(ok(s.db.all("SELECT m.*,u.username,u.display_name,u.email FROM organization_members m JOIN users u ON u.id=m.user_id WHERE m.organization_id=? ORDER BY m.id",vec![id.into()]).await?))
}
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
#[utoipa::path(delete,path="/orgs/{id}/members/{user}",tag="projects",params(spec::MemberPath),
    responses((status=200,description="Remove membership; owner required and last owner must remain",body=api::Envelope<crate::openapi::user_schemas::OkResult>),api::Errors),security(("bearer"=[])))]
pub(super) async fn delete_org_member(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((org, user)): Path<(i64, i64)>,
) -> Reply {
    mutate(&s, &h, org, user, Change::Remove).await
}
