use super::*;
use serde_json::json;

#[test]
fn filters_combine_state_text_and_exact_branch_names() {
    let rows = vec![
        json!({"iid":1,"title":"Fix review","description":"Unicode 文档","state":"opened","source_branch":"fix/ui","target_branch":"main"}),
        json!({"iid":2,"title":"Fix review","state":"merged","source_branch":"fix/ui","target_branch":"main"}),
        json!({"iid":3,"title":"Another","state":"opened","source_branch":"fix/ui-next","target_branch":"stable"}),
    ];
    assert_eq!(
        filter(&rows, "opened", "REVIEW", "fix/ui", "main"),
        vec![rows[0].clone()]
    );
    assert_eq!(filter(&rows, "all", "文档", "", "").len(), 1);
    assert_eq!(filter(&rows, "all", "", "fix/ui", "").len(), 2);
    assert!(filter(&rows, "closed", "", "", "").is_empty());
}

#[test]
fn user_ids_preserve_integer_precision_and_reject_invalid_values() {
    let values = vec![
        "9007199254740993".into(),
        "374500295142543361".into(),
        "invalid".into(),
    ];
    assert_eq!(ids(&values), vec![9007199254740993, 374500295142543361]);
    assert_eq!(
        strings(&json!(["9007199254740993", 42])),
        vec!["9007199254740993", "42"]
    );
}

#[test]
fn readiness_uses_server_gates_and_keeps_terminal_state() {
    assert_eq!(
        readiness(
            &json!({"state":"opened"}),
            &json!({"required":true,"mergeable":false})
        ),
        "Blocked"
    );
    assert_eq!(
        readiness(&json!({"state":"opened"}), &json!({"mergeable":true})),
        "Ready to merge"
    );
    assert_eq!(
        readiness(&json!({"state":"merged"}), &json!({"mergeable":false})),
        "Merged"
    );
    assert_eq!(
        readiness(&json!({"state":"closed"}), &Value::Null),
        "Closed"
    );
}

#[test]
fn user_labels_match_display_name_username_and_unknown_user_contract() {
    let users = json!([
        {"id":"42", "username":"reviewer", "display_name":"Reviewer"},
        {"id":"43", "username":"other", "display_name":""}
    ]);
    assert_eq!(user(&users, "42"), "Reviewer (@reviewer)");
    assert_eq!(user(&users, "43"), "@other");
    assert_eq!(user(&users, "44"), "#44");
}
