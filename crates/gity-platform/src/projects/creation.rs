use super::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

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
  s.git
    .validate_initial_branch(branch)
    .map_err(|_| AppError::bad_request("invalid default branch"))?;
  let tx = s.db.transaction().await?;
  // This no-op update acquires SQLite's write lock before checking org membership.
  tx.execute_raw(s.db.statement(
    "UPDATE organizations SET updated_at=updated_at WHERE id=?",
    vec![org_id.into()],
  ))
  .await?;
  let org = super::namespace::organization(&s, &tx, &actor, org_id, 30).await?;
  let full_path = format!("{}/{}", org["full_path"].as_str().unwrap_or_default(), path);
  let project_id = gity_core::new_id();
  let member_id = gity_core::new_id();
  let timestamp = chrono::Utc::now().naive_utc();
  let description = match b.get("description") {
    None | Some(Value::Null) => None,
    Some(Value::String(value)) => Some(value.clone()),
    _ => return Err(AppError::bad_request("description must be a string")),
  };
  let project = gity_entities::projects::ActiveModel {
    id: Set(project_id),
    organization_id: Set(org_id),
    name: Set(name.to_owned()),
    path_key: Set(path),
    full_path: Set(full_path),
    visibility: Set(visibility(&b)?.to_owned()),
    description: Set(description),
    default_branch: Set(branch.to_owned()),
    status: Set("active".to_owned()),
    deleted_at: Set(None),
    created_at: Set(timestamp),
    updated_at: Set(timestamp),
  }
  .insert(&tx)
  .await?;
  let member = gity_entities::project_members::ActiveModel {
    id: Set(member_id),
    project_id: Set(project_id),
    user_id: Set(actor.user_id),
    role: Set("owner".to_owned()),
    created_at: Set(timestamp),
    updated_at: Set(timestamp),
  }
  .insert(&tx)
  .await?;
  let row = access::project_record(project);
  let member_row = json!({
      "id": member.id,
      "project_id": member.project_id,
      "user_id": member.user_id,
      "role": member.role,
      "created_at": member.created_at.and_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
      "updated_at": member.updated_at.and_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
  });
  s.db
    .audit_snapshot(
      &tx,
      "project_members",
      &member_row,
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
    let compensation = s.db.transaction().await?;
    s.db
      .audit_snapshot(
        &compensation,
        "project_members",
        &member_row,
        "delete",
        Some(actor.user_id),
      )
      .await?;
    gity_entities::project_members::Entity::delete_many()
      .filter(gity_entities::project_members::Column::ProjectId.eq(project_id))
      .exec(&compensation)
      .await?;
    gity_entities::projects::Entity::delete_by_id(project_id)
      .exec(&compensation)
      .await?;
    compensation.commit().await?;
    tracing::warn!(project_id,error=%error,"project creation compensated after Git initialization failed");
    return Err(AppError::new(
      axum::http::StatusCode::SERVICE_UNAVAILABLE,
      "Git storage is unavailable; project creation was not completed",
    ));
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
