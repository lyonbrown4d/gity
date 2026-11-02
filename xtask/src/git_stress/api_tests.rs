use super::session_organization_id;
use serde_json::json;

#[test]
fn automatic_project_uses_the_sessions_personal_organization_identity() {
  let session = json!({"organization_id":"101","organizations":[
    {"id":"1","visibility":"public","role":"guest"},
    {"id":"101","visibility":"private","role":"owner"}
  ]});
  assert_eq!(session_organization_id(&session).unwrap(), 101);
  assert_eq!(
    session_organization_id(&json!({"organization_id":202})).unwrap(),
    202
  );
}

#[test]
fn automatic_project_refuses_missing_invalid_or_nonpositive_organization_identity() {
  for session in [
    json!({}),
    json!({"organization_id":null}),
    json!({"organization_id":""}),
    json!({"organization_id":"invalid"}),
    json!({"organization_id":"0"}),
    json!({"organization_id":-1}),
    json!({"organization_id":"9223372036854775808"}),
  ] {
    assert!(session_organization_id(&session).is_err(), "{session}");
  }
}
