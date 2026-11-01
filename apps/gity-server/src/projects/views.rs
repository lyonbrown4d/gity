use super::*;
pub(super) fn project_view(s: &AppState, mut row: Value) -> Value {
    row["uuid"] = json!(id(&row["id"]).unwrap_or_default().to_string());
    row["key"] = row["path_key"].clone();
    row["clone_http_url"] = json!(format!(
        "{}/{}.git",
        s.settings.base_url.trim_end_matches('/'),
        row["full_path"].as_str().unwrap_or_default()
    ));
    if row["description"].is_null() {
        row["description"] = json!("");
    }
    if row["deleted_at"].is_null() {
        row["deleted_at"] = json!("");
    }
    row
}
pub(super) fn organization_view(mut row: Value) -> Value {
    row["key"] = row["path_key"].clone();
    row["role"] = json!(row["current_role"].as_str().unwrap_or("guest"));
    row.as_object_mut().unwrap().remove("current_role");
    if row["description"].is_null() {
        row["description"] = json!("");
    }
    row
}
