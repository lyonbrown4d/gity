use super::*;
use sea_orm::{ConnectionTrait, FromQueryResult};

#[utoipa::path(
    post,path="/projects",tag="projects",
    description="Requires developer or higher organization membership; initializes the bare Git repository.",
    request_body=spec::CreateProject,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::Project>),api::Errors),
    security(("bearer"=[]))
)]
pub(super) async fn create(State(s): State<AppState>, h: HeaderMap, Json(b): Json<Value>) -> Reply {
    let actor = auth::principal(&s, &h).await?;
    let org_id = id(&b["organization_id"])?;
    let name = required(&b, "name")?;
    let path = slug(
        b["key"]
            .as_str()
            .or_else(|| b["path_key"].as_str())
            .or_else(|| b["path"].as_str())
            .unwrap_or(name),
    )?;
    let branch = b["default_branch"].as_str().unwrap_or("main");
    let tx = s.db.transaction().await?;
    tx.execute_raw(s.db.statement(
        "UPDATE organizations SET updated_at=updated_at WHERE id=?",
        vec![org_id.into()],
    ))
    .await?;
    let org = super::namespace::organization(&s, &tx, &actor, org_id, 30).await?;
    let full_path = format!("{}/{}", org["full_path"].as_str().unwrap_or_default(), path);
    let project_id = gity_core::new_id();
    let now = gity_core::now();
    tx.execute_raw(s.db.statement(
        "INSERT INTO projects(id,organization_id,name,path_key,full_path,visibility,description,default_branch,status,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?,?,?,?)",
        vec![project_id.into(),org_id.into(),name.into(),path.into(),full_path.into(),visibility(&b)?.into(),gity_core::sql_value(b["description"].clone())?,branch.into(),"active".into(),now.clone().into(),now.clone().into()],
    )).await?;
    tx.execute_raw(s.db.statement(
        "INSERT INTO project_members(id,project_id,user_id,role,created_at,updated_at) VALUES (?,?,?,?,?,?)",
        vec![gity_core::new_id().into(),project_id.into(),actor.user_id.into(),"owner".into(),now.clone().into(),now.into()],
    )).await?;
    let row = Value::find_by_statement(
        s.db.statement("SELECT * FROM projects WHERE id=?", vec![project_id.into()]),
    )
    .one(&tx)
    .await?
    .ok_or_else(|| AppError::not_found("project not found"))?;
    let member = Value::find_by_statement(s.db.statement(
        "SELECT * FROM project_members WHERE project_id=? AND user_id=?",
        vec![project_id.into(), actor.user_id.into()],
    ))
    .one(&tx)
    .await?
    .ok_or_else(|| AppError::not_found("project member not found"))?;
    s.db.audit_snapshot(
        &tx,
        "project_members",
        &member,
        "insert",
        Some(actor.user_id),
    )
    .await?;
    tx.commit().await?;
    if let Err(error) = s
        .git
        .init_bare(&crate::repository::repo(&row), branch)
        .await
    {
        s.db.delete("projects", project_id).await?;
        return Err(AppError::bad_request(error.to_string()));
    }
    crate::events::project_change(
        &s,
        &row,
        actor.user_id,
        crate::events::ProjectChange::Created,
    )
    .await?;
    Ok(ok(project_view(&s, row)))
}
