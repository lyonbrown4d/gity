use super::package_versions;
use serde_json::json;

#[test]
fn nuget_versions_read_the_nested_package_version() {
  let detail = json!({"versions":[
    {"version":{"version":"1.0.0","status":"default"},"files":[]},
    {"version":{"version":"1.1.0","status":"unlisted"},"files":[]}
  ]});
  let response = json!({"versions":package_versions(&detail)});
  assert_eq!(response, json!({"versions":["1.0.0","1.1.0"]}));
}
