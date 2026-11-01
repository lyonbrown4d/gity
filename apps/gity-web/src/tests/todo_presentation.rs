use super::*;
use serde_json::json;

#[test]
fn todo_actions_keep_local_deep_links_and_fall_back_for_external_urls() {
    let mut todo = json!({"organization_id":"42", "project_id":"9007199254740993", "action_url":"/app/projects/42/9007199254740993/issues/7"});
    assert_eq!(
        action_url(&todo),
        "/app/projects/42/9007199254740993/issues/7"
    );
    for action in [
        "https://external.example",
        "//external.example",
        "/admin/users",
        "",
    ] {
        todo["action_url"] = json!(action);
        assert_eq!(action_url(&todo), "/app/projects/42/9007199254740993");
    }
}

#[test]
fn todo_kind_labels_preserve_the_original_event_words() {
    assert_eq!(format_kind("merge_request_merged"), "Merge Request Merged");
    assert_eq!(format_kind("issue_assigned"), "Issue Assigned");
    assert_eq!(format_kind(""), "");
}
