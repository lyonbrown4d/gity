use super::*;
use serde_json::json;

#[test]
fn directory_scopes_preserve_namespace_isolation_and_exact_large_ids() {
  let all = projects(&json!([
      {"id":"9007199254740993","organization_id":"10","name":"Core API","key":"core","full_path":"team/core","default_branch":"main","visibility":"private"},
      {"id":"2","organization_id":"20","name":"Core API","key":"core","visibility":"public"},
      {"id":"3","organization_id":"10","name":"Site","key":"site","visibility":"public"}
  ]));
  let matches = projects_in_organization(&all, "10");
  assert_eq!(matches.len(), 2);
  assert_eq!(matches[0].id, "9007199254740993");
  assert!(projects_in_organization(&all, "").is_empty());
  assert!(!confirmation_matches(&matches[0], "core"));
  assert!(!confirmation_matches(&matches[0], " team/core"));
  assert!(confirmation_matches(&matches[0], "team/core"));
  assert!(!confirmation_matches(&all[1], ""));
}
#[test]
fn member_records_keep_user_identity_email_and_canonical_roles() {
  let rows = members(&json!([
      {"user_id":"9007199254740993","username":"Alice","email":"alice@example.test","role":"owner"},
      {"user_id":"2","username":"Bob","email":"bob@example.test","role":"developer"}
  ]));
  assert_eq!(rows[0].user_id, "9007199254740993");
  assert_eq!(rows[0].email, "alice@example.test");
  assert_eq!(rows[1].role, "developer");
}
