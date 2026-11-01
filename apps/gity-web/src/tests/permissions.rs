use super::*;
use serde_json::json;

#[test]
fn explicit_denial_overrides_capability_and_role() {
  let value = json!({"actions":{"project.repository.push":false},"capabilities":{"repository_push":true},"role_level":50});
  assert!(!allowed(
    &value,
    "project.repository.push",
    "repository_push",
    30
  ));
  assert!(!allowed(
    &json!({"capabilities":{"repository_push":false},"role_level":50}),
    "project.repository.push",
    "repository_push",
    30
  ));
}

#[test]
fn server_capabilities_and_role_thresholds_are_supported_without_granting_unknown_roles() {
  assert!(allowed(
    &json!({"capabilities":{"repository_push":true}}),
    "project.repository.push",
    "repository_push",
    30
  ));
  assert!(allowed(
    &json!({"role_level":40}),
    "project.repository.admin",
    "repository_admin",
    40
  ));
  assert!(!allowed(
    &json!({"role_level":30}),
    "project.repository.admin",
    "repository_admin",
    40
  ));
  assert!(!allowed(
    &Value::Null,
    "project.repository.push",
    "repository_push",
    30
  ));
}
