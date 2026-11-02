use super::*;
use crate::openapi::{project_schemas as spec, schemas as api};

pub(super) async fn org_access(
  s: &AppState,
  h: &HeaderMap,
  org: i64,
  min: i64,
) -> Result<Value, AppError> {
  let actor = auth::principal(s, h).await?;
  let (model, role) = s.projects.organization(&actor, org, min).await?;
  Ok(organization_view(super::namespace::organization_record(
    model, &role,
  )))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    get,path="/orgs",tag="projects",
    description="Authenticated users see public/internal organizations and their memberships.",
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Organization>>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn orgs(State(s): State<AppState>, h: HeaderMap) -> Reply {
  let p = auth::principal(&s, &h).await?;
  let organizations = s.projects.organizations().await?;
  let roles = s.identity.organization_roles(p.user_id).await?;
  let mut items = Vec::new();
  for organization in organizations {
    let role = roles
      .get(&organization.id)
      .copied()
      .unwrap_or(gity_identity::MembershipRole::None);
    if p.is_super_admin
      || organization.visibility != "private"
      || role != gity_identity::MembershipRole::None
    {
      let current_role = if p.is_super_admin {
        "owner"
      } else {
        match role {
          gity_identity::MembershipRole::Owner => "owner",
          gity_identity::MembershipRole::Maintainer => "maintainer",
          gity_identity::MembershipRole::Developer => "developer",
          gity_identity::MembershipRole::Reporter => "reporter",
          _ => "guest",
        }
      };
      items.push(organization_view(super::namespace::organization_record(
        organization,
        current_role,
      )));
    }
  }
  Ok(ok(items))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    get,path="/orgs/{id}",tag="projects",
    description="Private organization reads require membership.",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Organization>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn org(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i64>) -> Reply {
  Ok(ok(org_access(&s, &h, id, 0).await?))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    post,path="/orgs",tag="projects",
    description="Creates an organization owned by the current authenticated user.",
    request_body=spec::CreateOrganization,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Organization>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn create_org(
  State(s): State<AppState>,
  h: HeaderMap,
  Json(b): Json<Value>,
) -> Reply {
  let actor = auth::principal(&s, &h).await?;
  let name = required(&b, "name")?;
  let path_key = slug(
    b["key"]
      .as_str()
      .or_else(|| b["path_key"].as_str())
      .or_else(|| b["path"].as_str())
      .unwrap_or(name),
  )?;
  let model = s
    .projects
    .create_organization(
      &actor,
      gity_projects::CreateOrganization {
        name: name.into(),
        path_key,
        description: description(&b)?,
        visibility: visibility(&b)?.into(),
      },
    )
    .await?;
  Ok(ok(organization_view(
    super::namespace::organization_record(model, "owner"),
  )))
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    patch,path="/orgs/{id}",tag="projects",
    description="Requires organization owner.",
    params(api::ProjectPath),
    request_body=spec::UpdateOrganization,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Organization>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn update_org(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  super::namespace::update(&s, &h, id, &b).await
}
#[gity_api_macros::application_operation(prefix = "/api/v1")]
#[utoipa::path(
    delete,path="/orgs/{id}",tag="projects",
    description="Requires organization owner.",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<crate::openapi::user_schemas::OkResult>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn delete_org(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(id): Path<i64>,
) -> Reply {
  let actor = auth::principal(&s, &h).await?;
  s.projects.delete_organization(&actor, id).await?;
  Ok(ok(json!({"ok":true})))
}
