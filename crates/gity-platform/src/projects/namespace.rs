use super::*;
pub(super) async fn update(s: &AppState, h: &HeaderMap, id: i64, body: &Value) -> Reply {
  let actor = auth::principal(s, h).await?;
  let command = gity_projects::OrganizationChanges {
    name: body
      .get("name")
      .map(|_| required(body, "name").map(str::to_owned))
      .transpose()?,
    path_key: body
      .get("key")
      .map(|value| slug(value.as_str().unwrap_or_default()))
      .transpose()?,
    description: body
      .get("description")
      .map(|_| description(body))
      .transpose()?,
    visibility: body
      .get("visibility")
      .map(|_| visibility(body).map(str::to_owned))
      .transpose()?,
  };
  let (model, role) = s.projects.update_organization(&actor, id, command).await?;
  Ok(ok(organization_view(organization_record(model, &role))))
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
      "current_role": if role.is_empty() {"guest"} else {role},
  })
}
