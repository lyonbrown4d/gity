use super::*;
use crate::openapi::{project_schemas as spec, schemas as api};

pub(super) async fn org_access(
    s: &AppState,
    h: &HeaderMap,
    org: i64,
    min: i64,
) -> Result<Value, AppError> {
    let p = auth::principal(s, h).await?;
    let row =
        s.db.one("SELECT o.*,(SELECT role FROM organization_members WHERE organization_id=o.id AND user_id=?) AS current_role FROM organizations o WHERE o.id=?", vec![p.user_id.into(),org.into()])
            .await?
            .ok_or_else(|| AppError::not_found("organization not found"))?;
    let level = access::organization_level(s, &p, org).await?;
    let allowed = if min == 0 {
        row["visibility"] == "public" || row["visibility"] == "internal" || level > 0
    } else {
        level >= min
    };
    if !allowed {
        return Err(AppError::forbidden("insufficient organization permissions"));
    }
    let mut row = row;
    if p.is_super_admin {
        row["current_role"] = json!("owner");
    }
    Ok(organization_view(row))
}
#[utoipa::path(
    get,path="/orgs",tag="projects",
    description="Authenticated users see public/internal organizations and their memberships.",
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::Organization>>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn orgs(State(s): State<AppState>, h: HeaderMap) -> Reply {
    let p = auth::principal(&s, &h).await?;
    let items = if p.is_super_admin {
        s.db.all(
            "SELECT o.*,'owner' AS current_role FROM organizations o ORDER BY id",
            vec![],
        )
        .await?
    } else {
        s.db.all("SELECT DISTINCT o.*,m.role AS current_role FROM organizations o LEFT JOIN organization_members m ON m.organization_id=o.id AND m.user_id=? WHERE m.id IS NOT NULL OR o.visibility IN ('public','internal') ORDER BY o.id",vec![p.user_id.into()]).await?
    };
    Ok(ok(items
        .into_iter()
        .map(organization_view)
        .collect::<Vec<_>>()))
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
    let org=s.db.insert("organizations",json!({"name":name,"path_key":path,"full_path":path,"visibility":visibility(&b)?,"description":b.get("description")}).as_object().unwrap().clone()).await?;
    let org_id = id(&org["id"])?;
    if let Err(error) =
        s.db.insert(
            "organization_members",
            json!({"organization_id":org_id,"user_id":actor.user_id,"role":"owner"})
                .as_object()
                .unwrap()
                .clone(),
        )
        .await
    {
        s.db.delete("organizations", org_id).await?;
        return Err(error);
    }
    let mut org = org;
    org["current_role"] = json!("owner");
    Ok(ok(organization_view(org)))
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
    s.db.delete("organizations", id).await?;
    Ok(ok(json!({"ok":true})))
}
