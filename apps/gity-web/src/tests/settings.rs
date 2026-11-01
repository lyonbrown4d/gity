use super::*;
#[test]
fn member_filters_cover_role_and_case_insensitive_identity_without_losing_large_ids() {
    let members = json!([{"user_id":"9223372036854775806","username":"Alice","role":"owner"},{"user_id":"2","username":"Bob","email":"Bob@example.test","role":"developer"}]);
    assert_eq!(
        filtered(&members, " alice ", "all")[0]["user_id"],
        "9223372036854775806"
    );
    assert!(filtered(&members, "alice", "developer").is_empty());
    assert_eq!(filtered(&members, "EXAMPLE", "developer").len(), 1);
}
#[test]
fn token_requests_require_scopes_and_use_explicit_utc_expiry_only_when_supplied() {
    assert!(
        token_body(
            " ".into(),
            String::new(),
            String::new(),
            vec!["read_repository".into()]
        )
        .is_err()
    );
    assert!(token_body("name".into(), String::new(), String::new(), vec![]).is_err());
    let value = token_body(
        " name ".into(),
        " bot ".into(),
        "2030-05-07".into(),
        vec!["read_repository".into()],
    )
    .unwrap();
    assert_eq!(value["expires_at"], "2030-05-07T00:00:00Z");
    assert_eq!(value["username"], "bot");
    assert!(
        token_body(
            "name".into(),
            String::new(),
            String::new(),
            vec!["read_repository".into()]
        )
        .unwrap()
        .get("expires_at")
        .is_none()
    );
}
