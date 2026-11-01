use super::*;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseTransaction, EntityTrait,
  IntoActiveModel, QueryFilter, Set,
};

pub(super) async fn organization(
  _s: &AppState,
  tx: &DatabaseTransaction,
  actor: &gity_core::Principal,
  id: i64,
  minimum: i64,
) -> Result<Value, AppError> {
  let org = gity_entities::organizations::Entity::find_by_id(id)
    .one(tx)
    .await?
    .ok_or_else(|| AppError::not_found("organization not found"))?;
  let role = gity_entities::organization_members::Entity::find()
    .filter(gity_entities::organization_members::Column::OrganizationId.eq(id))
    .filter(gity_entities::organization_members::Column::UserId.eq(actor.user_id))
    .one(tx)
    .await?
    .map(|member| member.role)
    .unwrap_or_default();
  let current_role = if actor.is_super_admin { "owner" } else { &role };
  if !actor.is_super_admin && access::role_level(current_role) < minimum {
    return Err(AppError::forbidden("insufficient organization permissions"));
  }
  Ok(organization_record(org, current_role))
}

pub(super) async fn update(s: &AppState, h: &HeaderMap, id: i64, body: &Value) -> Reply {
  let actor = auth::principal(s, h).await?;
  let key = body
    .get("key")
    .map(|key| slug(key.as_str().unwrap_or_default()))
    .transpose()?;
  if body.get("name").is_some() {
    required(body, "name")?;
  }
  if body.get("visibility").is_some() {
    visibility(body)?;
  }
  let tx = s.db.transaction().await?;
  // These no-op updates acquire write locks before path/member reads on SQLite.
  if key.is_some() {
    tx.execute_raw(
      s.db
        .statement("UPDATE organizations SET updated_at=updated_at", vec![]),
    )
    .await?;
  } else {
    tx.execute_raw(s.db.statement(
      "UPDATE organizations SET updated_at=updated_at WHERE id=?",
      vec![id.into()],
    ))
    .await?;
  }
  let current = organization(s, &tx, &actor, id, 50).await?;
  if let Some(key) = &key {
    let old = current["full_path"].as_str().unwrap_or_default();
    let next = current["full_path"]
      .as_str()
      .and_then(|path| path.rsplit_once('/'))
      .map(|(parent, _)| format!("{parent}/{key}"))
      .unwrap_or_else(|| key.clone());
    if old != next {
      rename(&tx, old, &next).await?;
    }
  }
  let mut org = gity_entities::organizations::Entity::find_by_id(id)
    .one(&tx)
    .await?
    .ok_or_else(|| AppError::not_found("organization not found"))?
    .into_active_model();
  if let Some(name) = body.get("name") {
    org.name = Set(name.as_str().unwrap_or_default().to_owned());
  }
  if let Some(description) = body.get("description") {
    org.description = Set(match description {
      Value::Null => None,
      Value::String(value) => Some(value.clone()),
      _ => return Err(AppError::bad_request("description must be a string")),
    });
  }
  if let Some(visibility) = body.get("visibility") {
    org.visibility = Set(visibility.as_str().unwrap_or("private").to_owned());
  }
  if let Some(key) = key {
    org.path_key = Set(key);
  }
  org.updated_at = Set(chrono::Utc::now().naive_utc());
  org.update(&tx).await?;
  let row = organization(s, &tx, &actor, id, 50).await?;
  tx.commit().await?;
  Ok(ok(organization_view(row)))
}

async fn rename(tx: &DatabaseTransaction, old: &str, next: &str) -> Result<(), AppError> {
  let prefix = format!("{old}/");
  let organizations = gity_entities::organizations::Entity::find()
    .filter(
      Condition::any()
        .add(gity_entities::organizations::Column::FullPath.eq(old))
        .add(gity_entities::organizations::Column::FullPath.starts_with(&prefix)),
    )
    .all(tx)
    .await?
    .into_iter()
    .filter(|row| row.full_path == old || row.full_path.starts_with(&prefix))
    .collect::<Vec<_>>();
  let projects = gity_entities::projects::Entity::find()
    .filter(
      Condition::any()
        .add(gity_entities::projects::Column::FullPath.eq(old))
        .add(gity_entities::projects::Column::FullPath.starts_with(&prefix)),
    )
    .all(tx)
    .await?
    .into_iter()
    .filter(|row| row.full_path == old || row.full_path.starts_with(&prefix))
    .collect::<Vec<_>>();
  let timestamp = chrono::Utc::now().naive_utc();
  for row in organizations {
    let suffix = row
      .full_path
      .strip_prefix(old)
      .unwrap_or_default()
      .to_owned();
    let mut active = row.into_active_model();
    active.full_path = Set(format!("{next}{suffix}"));
    active.updated_at = Set(timestamp);
    active.update(tx).await?;
  }
  for row in projects {
    let suffix = row
      .full_path
      .strip_prefix(old)
      .unwrap_or_default()
      .to_owned();
    let mut active = row.into_active_model();
    active.full_path = Set(format!("{next}{suffix}"));
    active.updated_at = Set(timestamp);
    active.update(tx).await?;
  }
  Ok(())
}

pub(super) fn organization_record(org: gity_entities::organizations::Model, role: &str) -> Value {
  json!({
      "id": org.id,
      "name": org.name,
      "path_key": org.path_key,
      "full_path": org.full_path,
      "description": org.description,
      "visibility": org.visibility,
      "created_at": org.created_at.and_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
      "updated_at": org.updated_at.and_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
      "current_role": role,
  })
}
