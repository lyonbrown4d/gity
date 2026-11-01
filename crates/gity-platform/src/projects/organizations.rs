use super::*;
use crate::openapi::{project_schemas as spec, schemas as api};
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};

pub(super) async fn org_access(
  s: &AppState,
  h: &HeaderMap,
  org: i64,
  min: i64,
) -> Result<Value, AppError> {
  let p = auth::principal(s, h).await?;
  let organization = gity_entities::organizations::Entity::find_by_id(org)
    .one(&s.db.connection)
    .await?
    .ok_or_else(|| AppError::not_found("organization not found"))?;
  let role = s.identity.organization_role(org, p.user_id).await?;
  let level = access::organization_level(s, &p, org).await?;
  let allowed = if min == 0 {
    organization.visibility == "public" || organization.visibility == "internal" || level > 0
  } else {
    level >= min
  };
  if !allowed {
    return Err(AppError::forbidden("insufficient organization permissions"));
  }
  let current_role = if p.is_super_admin {
    "owner"
  } else {
    match role {
      gity_identity::MembershipRole::Owner => "owner",
      gity_identity::MembershipRole::Maintainer => "maintainer",
      gity_identity::MembershipRole::Developer => "developer",
      gity_identity::MembershipRole::Reporter => "reporter",
      gity_identity::MembershipRole::Guest => "guest",
      _ => "guest",
    }
  };
  Ok(organization_view(super::namespace::organization_record(
    organization,
    current_role,
  )))
}
#[utoipa::path(
    get,path="/orgs",tag="projects",
    description="Authenticated users see public/internal organizations and their memberships.",
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Organization>>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn orgs(State(s): State<AppState>, h: HeaderMap) -> Reply {
  let p = auth::principal(&s, &h).await?;
  let organizations = gity_entities::organizations::Entity::find()
    .order_by_asc(gity_entities::organizations::Column::Id)
    .all(&s.db.connection)
    .await?;
  let roles = s.identity.organization_roles(p.user_id).await?;
  let mut items = Vec::new();
  for organization in organizations {
    let role = roles
      .get(&organization.id)
      .copied()
      .unwrap_or(gity_identity::MembershipRole::None);
    let accessible = p.is_super_admin
      || organization.visibility == "public"
      || organization.visibility == "internal"
      || role != gity_identity::MembershipRole::None;
    if accessible {
      let current_role = if p.is_super_admin {
        "owner"
      } else {
        match role {
          gity_identity::MembershipRole::Owner => "owner",
          gity_identity::MembershipRole::Maintainer => "maintainer",
          gity_identity::MembershipRole::Developer => "developer",
          gity_identity::MembershipRole::Reporter => "reporter",
          gity_identity::MembershipRole::Guest => "guest",
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
  let path = slug(
    b["key"]
      .as_str()
      .or_else(|| b["path_key"].as_str())
      .or_else(|| b["path"].as_str())
      .unwrap_or(name),
  )?;
  let description = match b.get("description") {
    None | Some(Value::Null) => None,
    Some(Value::String(value)) => Some(value.clone()),
    _ => return Err(AppError::bad_request("description must be a string")),
  };
  let org_id = gity_core::new_id();
  let now = chrono::Utc::now().naive_utc();
  let tx = s.db.transaction().await?;
  let organization = gity_entities::organizations::ActiveModel {
    id: Set(org_id),
    name: Set(name.to_owned()),
    path_key: Set(path.clone()),
    full_path: Set(path),
    description: Set(description),
    visibility: Set(visibility(&b)?.to_owned()),
    created_at: Set(now),
    updated_at: Set(now),
  }
  .insert(&tx)
  .await?;
  gity_entities::organization_members::ActiveModel {
    id: Set(gity_core::new_id()),
    organization_id: Set(org_id),
    user_id: Set(actor.user_id),
    role: Set("owner".to_owned()),
    created_at: Set(now),
    updated_at: Set(now),
  }
  .insert(&tx)
  .await?;
  tx.commit().await?;
  Ok(ok(organization_view(
    super::namespace::organization_record(organization, "owner"),
  )))
}
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
  org_access(&s, &h, id, 50).await?;
  gity_entities::organizations::Entity::delete_by_id(id)
    .exec(&s.db.connection)
    .await?;
  Ok(ok(json!({"ok":true})))
}
