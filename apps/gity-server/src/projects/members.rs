use super::*;
use crate::openapi::{project_schemas as spec, schemas as api};

#[utoipa::path(
    get,path="/projects/{id}/members",tag="projects",
    description="Requires project read access.",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::ProjectMember>>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn members(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i64>) -> Reply {
    access::project_action(&s, &h, id, access::ProjectAction::RepositoryRead).await?;
    Ok(ok(s.db.all("SELECT m.*,u.username,u.display_name FROM project_members m JOIN users u ON u.id=m.user_id WHERE m.project_id=? ORDER BY m.id",vec![id.into()]).await?))
}
pub(super) fn member_role(b: &Value) -> Result<&str, AppError> {
    let role = required(b, "role")?;
    if access::role_level(role) == 0 {
        return Err(AppError::bad_request("invalid membership role"));
    }
    Ok(role)
}
#[utoipa::path(
    post,path="/projects/{id}/members",tag="projects",
    description="Requires maintainer. Granting/changing owner membership requires owner.",
    params(api::ProjectPath),
    request_body=spec::AddMember,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ProjectMember>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn add_member(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(project): Path<i64>,
    Json(b): Json<Value>,
) -> Reply {
    set_member(State(s), h, Path((project, id(&b["user_id"])?)), Json(b)).await
}
#[utoipa::path(
    patch,path="/projects/{id}/members/{user}",tag="projects",
    description="Requires maintainer; owner membership is protected.",
    params(spec::MemberPath),
    request_body=spec::SetMember,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ProjectMember>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn set_member(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((project, user)): Path<(i64, i64)>,
    Json(b): Json<Value>,
) -> Reply {
    access::project(&s, &h, project, 40).await?;
    let role = member_role(&b)?;
    let p = auth::principal(&s, &h).await?;
    let proj = access::project(&s, &h, project, 40).await?;
    if role == "owner" && access::level(&s, &p, &proj).await? < 50 {
        return Err(AppError::forbidden("owner role required"));
    }
    let row = if let Some(row) =
        s.db.one(
            "SELECT * FROM project_members WHERE project_id=? AND user_id=?",
            vec![project.into(), user.into()],
        )
        .await?
    {
        if row["role"] == "owner" && access::level(&s, &p, &proj).await? < 50 {
            return Err(AppError::forbidden("owner role required"));
        }
        s.db.update(
            "project_members",
            id(&row["id"])?,
            json!({"role":role}).as_object().unwrap().clone(),
        )
        .await?
    } else {
        s.db.insert(
            "project_members",
            json!({"project_id":project,"user_id":user,"role":role})
                .as_object()
                .unwrap()
                .clone(),
        )
        .await?
    };
    Ok(ok(row))
}
#[utoipa::path(
    delete,path="/projects/{id}/members/{user}",tag="projects",
    description="Requires maintainer; removing an owner requires owner.",
    params(spec::MemberPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<crate::openapi::user_schemas::OkResult>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn delete_member(
    State(s): State<AppState>,
    h: HeaderMap,
    Path((project, user)): Path<(i64, i64)>,
) -> Reply {
    access::project(&s, &h, project, 40).await?;
    if let Some(row) =
        s.db.one(
            "SELECT * FROM project_members WHERE project_id=? AND user_id=?",
            vec![project.into(), user.into()],
        )
        .await?
    {
        if row["role"] == "owner" {
            let actor = auth::principal(&s, &h).await?;
            let proj = access::project(&s, &h, project, 40).await?;
            if access::level(&s, &actor, &proj).await? < 50 {
                return Err(AppError::forbidden("owner role required"));
            }
        }
        s.db.delete("project_members", id(&row["id"])?).await?;
    }
    Ok(ok(json!({"ok":true})))
}
