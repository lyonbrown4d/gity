use super::*;

#[test]
fn lists_nested_module_versions_and_hides_unlisted_versions() {
  let detail = json!({
    "versions": [
      {"version":{"version":"v1.2.0","status":"default"}},
      {"version":{"version":"v1.0.0","status":"default"}},
      {"version":{"version":"v1.1.0","status":"unlisted"}}
    ]
  });

  assert_eq!(listed_versions(&detail), ["v1.0.0", "v1.2.0"]);
}

#[test]
fn decodes_uppercase_module_version_paths() {
  assert_eq!(decode_go_version("v1.0.0-!r!c1").unwrap(), "v1.0.0-RC1");
}
