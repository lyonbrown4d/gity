use super::*;
use std::collections::BTreeSet;

/// Route coverage survives deletion of the old implementation. Parameter names
/// may differ, but a legacy method and path shape must remain documented.
#[test]
fn generated_openapi_preserves_every_recorded_legacy_operation() {
  let document = serde_json::to_value(document()).unwrap();
  let normalize = regex::Regex::new(r"\{[^}]+\}").unwrap();
  let mut actual = BTreeSet::new();
  for (path, item) in document["paths"].as_object().unwrap() {
    for method in ["get", "post", "put", "patch", "delete"] {
      if item.get(method).is_some() {
        actual.insert((
          method.to_owned(),
          normalize.replace_all(path, "{}").into_owned(),
        ));
      }
    }
  }
  let baseline = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../contracts/legacy-http-routes.tsv"
  ));
  let mut count = 0;
  for line in baseline
    .lines()
    .filter(|line| !line.starts_with('#') && !line.is_empty())
  {
    let (method, path) = line.split_once('\t').expect("recorded method and path");
    count += 1;
    assert!(
      actual.contains(&(
        method.to_owned(),
        normalize.replace_all(path, "{}").into_owned()
      )),
      "missing legacy operation: {method} {path}"
    );
  }
  assert_eq!(
    count, 234,
    "baseline changes require an explicit compatibility review"
  );
}
