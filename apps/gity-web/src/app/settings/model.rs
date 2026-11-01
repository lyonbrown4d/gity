use crate::api;
use serde_json::{Value, json};
pub(super) const ROLES: [&str; 5] = ["guest", "reporter", "developer", "maintainer", "owner"];
pub(super) const SCOPES: [&str; 6] = [
    "read_repository",
    "write_repository",
    "read_package",
    "write_package",
    "read_api",
    "write_api",
];
pub(super) fn role_description(role: &str) -> &'static str {
    match role {
        "guest" => "Can view and comment.",
        "reporter" => "Can inspect code and issues.",
        "developer" => "Can push branches and open merge requests.",
        "maintainer" => "Can merge and manage repository settings.",
        "owner" => "Full project administration.",
        _ => "Project member role.",
    }
}
pub(super) fn filtered(members: &Value, search: &str, role: &str) -> Vec<Value> {
    let search = search.trim().to_lowercase();
    api::list(members)
        .into_iter()
        .filter(|m| {
            (role == "all" || m["role"] == role)
                && ["username", "display_name", "email", "user_id"]
                    .iter()
                    .any(|key| api::text(m, key).to_lowercase().contains(&search))
        })
        .collect()
}
pub(super) fn token_body(
    name: String,
    username: String,
    expires: String,
    scopes: Vec<String>,
) -> Result<Value, &'static str> {
    if name.trim().is_empty() || scopes.is_empty() {
        return Err("Token name and scopes are required.");
    }
    let mut value = json!({"name":name.trim(),"username":username.trim(),"scopes":scopes});
    if !expires.is_empty() {
        value["expires_at"] = json!(format!("{expires}T00:00:00Z"));
    }
    Ok(value)
}
#[cfg(test)]
#[path = "../../tests/settings.rs"]
mod tests;
