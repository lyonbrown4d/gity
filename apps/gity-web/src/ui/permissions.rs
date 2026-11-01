use serde_json::Value;

pub fn role_label(value: &Value) -> String {
    if value["role_level"]
        .as_u64()
        .is_some_and(|level| level >= 100)
    {
        "super-admin".into()
    } else {
        let role = crate::api::text(value, "role");
        if role.is_empty() {
            "guest".into()
        } else {
            role
        }
    }
}

/// Respect the server's explicit decision before considering its effective role.
pub fn allowed(value: &Value, action: &str, capability: &str, minimum_level: u64) -> bool {
    value["actions"][action]
        .as_bool()
        .or_else(|| value["capabilities"][capability].as_bool())
        .unwrap_or_else(|| {
            value["role_level"]
                .as_u64()
                .is_some_and(|level| level >= minimum_level)
        })
}

#[cfg(test)]
#[path = "../tests/permissions.rs"]
mod tests;
