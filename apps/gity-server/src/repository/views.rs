use serde_json::{Value, json};

/// Keep existing browser and external API field names while retaining the
/// richer native Git metadata for the Rust frontend.
pub(super) fn present(action: &str, id: i64, branch: &str, mut value: Value) -> Value {
    match action {
        "branches" => {
            if let Some(rows) = value.as_array_mut() {
                for row in rows {
                    row["repository_id"] = json!(id);
                    row["is_protected"] = row["protected"].clone();
                    row["last_commit_sha"] = row["hash"].clone();
                    if row["protection"].is_object() {
                        row["protection"] = protection(row["protection"].take());
                    }
                }
            }
        }
        "commits" => {
            if let Some(rows) = value.as_array_mut() {
                for row in rows {
                    row["repository_id"] = json!(id);
                    row["branch_name"] = json!(branch);
                    row["commit_sha"] = row["hash"].clone();
                    row["author_user_id"] = row["author_name"].clone();
                    row["created_at"] = row["committed_at"].clone();
                }
            }
        }
        "tree" => {
            if let Some(rows) = value.as_array_mut() {
                for row in rows {
                    row["kind"] = row["type"].clone();
                    row["oid"] = row["mode"].clone();
                }
            }
        }
        "blob" | "readme" if value.is_object() => {
            value["is_binary"] = json!(value["encoding"] == "base64");
        }
        _ => {}
    }
    value
}

/// Convert only database protection flags; preserve arbitrary user content.
pub(super) fn protection(mut row: Value) -> Value {
    for key in [
        "require_merge_request",
        "require_pipeline_success",
        "allow_force_push",
        "allow_delete",
    ] {
        if let Some(value) = row.get_mut(key)
            && let Some(flag) = value.as_i64()
        {
            *value = json!(flag != 0);
        }
    }
    row
}
