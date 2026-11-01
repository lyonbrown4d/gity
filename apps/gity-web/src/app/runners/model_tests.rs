use super::*;

#[test]
fn runner_status_and_tags_keep_offline_fallback_and_whitespace_rules() {
    let runner = Runner::from(
        &json!({"id":"9007199254740993","name":"","status":"ONLINE","active":false,"tags":" linux, ,amd64, "}),
    );
    assert_eq!(runner.id, "9007199254740993");
    assert_eq!(runner.name, "runner");
    assert!(runner.online);
    assert!(!runner.active);
    assert_eq!(runner.tags, ["linux", "amd64"]);
    assert!(!Runner::from(&json!({"status":"unknown"})).online);
    let stats = Stats::from_runners(&[runner, Runner::from(&json!({"active":true}))]);
    assert_eq!(
        (stats.registered, stats.online, stats.offline, stats.active),
        (2, 1, 1, 1)
    );
}

#[test]
fn registration_trims_fields_and_rejects_blank_runner_name() {
    assert!(registration(" \n", "description", "linux").is_err());
    assert_eq!(
        registration(" runner ", " description ", " linux,amd64 ").unwrap(),
        json!({"name":"runner","description":"description","tags":"linux,amd64"})
    );
}

#[test]
fn variable_submission_normalizes_key_but_preserves_secret_value_bytes() {
    let value = "  fixture-secret  ";
    let body = variable_body(" deploy_token ", value, true, true).unwrap();
    assert_eq!(
        body,
        json!({"key":"DEPLOY_TOKEN","value":value,"masked":true,"protected":true})
    );
    assert!(variable_body(" ", value, false, false).is_err());
    assert!(variable_body("KEY", "short", true, false).is_err());
    assert!(variable_body("KEY", "x", false, false).is_ok());
}

#[test]
fn variable_guidance_distinguishes_masking_from_protected_execution_scope() {
    assert!(variable_guidance(true, true).contains("protected ref"));
    assert!(variable_guidance(true, false).contains("8 characters"));
    assert!(variable_exposure(false, true).contains("avoid storing raw secrets"));
    assert!(variable_exposure(false, false).contains("non-secret"));
    assert_eq!(
        role_label(&json!({"role":"owner","role_level":100})),
        "super-admin"
    );
    assert_eq!(
        role_label(&json!({"role":"reporter","role_level":20})),
        "reporter"
    );
}
