use super::build;

#[test]
fn registered_routes_are_present_in_the_openapi_document() {
  let document = serde_json::to_value(crate::openapi::document()).unwrap();
  let missing = inventory::iter::<super::super::OperationRegistration>()
    .filter_map(|registration| {
      let path = super::join_path(registration.prefix, registration.path);
      let route = format!("{} {path}", registration.method);
      let pointer = format!("/paths/{}", path.replace('~', "~0").replace('/', "~1"));
      let method = registration.method.to_ascii_lowercase();
      if document
        .pointer(&pointer)
        .and_then(|item| item.get(method))
        .is_none()
      {
        Some(format!("{route} ({})", registration.handler))
      } else {
        None
      }
    })
    .collect::<Vec<_>>();
  assert!(missing.is_empty(), "undocumented operations: {missing:#?}");
}

#[test]
fn production_catalog_has_unique_names_and_valid_schemas() {
  let operations = build().expect("all registered handlers have catalog contracts");
  assert!(!operations.is_empty());

  let mut names = std::collections::HashSet::new();
  let mut errors = Vec::new();
  for operation in operations {
    assert!(names.insert(operation.contract.name.clone()));
    jsonschema::validator_for(&operation.contract.input_schema).unwrap_or_else(|error| {
      panic!(
        "invalid input schema for {}: {error}",
        operation.contract.name
      )
    });
    if let Err(error) = jsonschema::validator_for(&operation.contract.output_schema) {
      errors.push(format!("{} output: {error}", operation.contract.name));
    }
  }
  assert!(errors.is_empty(), "invalid operation schemas: {errors:#?}");
}

#[test]
fn classified_handlers_and_aliases_match_the_operation_inventory() {
  let business: serde_json::Value = serde_json::from_str(include_str!("../catalog_business.json"))
    .expect("business operation manifest is valid JSON");
  let exclusions: serde_json::Value =
    serde_json::from_str(include_str!("../catalog_exclusions.json"))
      .expect("operation exclusions manifest is valid JSON");
  let registrations = inventory::iter::<super::super::OperationRegistration>().collect::<Vec<_>>();

  for handler in business["expected_business"].as_array().unwrap() {
    assert_registered(handler.as_str().unwrap(), &registrations);
  }
  for category in exclusions["excluded"].as_object().unwrap().values() {
    assert!(!category["reason"].as_str().unwrap().is_empty());
    for handler in category["handlers"].as_array().unwrap() {
      assert_not_registered(handler.as_str().unwrap(), &registrations);
    }
  }
  for (source, adapter) in business["business_adapters"].as_object().unwrap() {
    assert!(source.starts_with("gity-package-registry/"));
    assert_registered(adapter.as_str().unwrap(), &registrations);
  }
  for aliases in [
    include_str!("../catalog_aliases_a.json"),
    include_str!("../catalog_aliases_b.json"),
    include_str!("../catalog_aliases_c.json"),
  ] {
    let aliases: serde_json::Value =
      serde_json::from_str(aliases).expect("operation alias manifest is valid JSON");
    for alias in aliases["canonical_aliases"].as_array().unwrap() {
      assert_canonical_alias(alias, &registrations);
    }
  }
}

fn assert_canonical_alias(
  alias: &serde_json::Value,
  registrations: &[&super::super::OperationRegistration],
) {
  let module_suffix = module_suffix(alias["handler"].as_str().unwrap());
  let canonical = alias["canonical_prefix"].as_str().unwrap();
  assert!(
    registrations
      .iter()
      .any(|item| item.handler.ends_with(&module_suffix) && item.prefix == canonical)
  );
  for alias_prefix in alias["alias_prefixes"].as_array().unwrap() {
    let alias_prefix = alias_prefix.as_str().unwrap();
    assert!(
      !registrations
        .iter()
        .any(|item| item.handler.ends_with(&module_suffix) && item.prefix == alias_prefix)
    );
  }
}

fn assert_registered(handler: &str, registrations: &[&super::super::OperationRegistration]) {
  let suffix = module_suffix(handler);
  assert!(
    registrations
      .iter()
      .any(|item| item.handler.ends_with(&suffix)),
    "business handler is not annotated: {handler}"
  );
}

fn assert_not_registered(handler: &str, registrations: &[&super::super::OperationRegistration]) {
  let suffix = module_suffix(handler);
  assert!(
    registrations
      .iter()
      .all(|item| !item.handler.ends_with(&suffix)),
    "classified exclusion is unexpectedly registered: {handler}"
  );
}

fn module_suffix(handler: &str) -> String {
  let (source, function) = handler.split_once('#').expect("file#function entry");
  let module = if source == "features/merge_execution.rs" {
    "features::merge_requests::execution".into()
  } else {
    let mut parts = source
      .trim_end_matches(".rs")
      .split('/')
      .collect::<Vec<_>>();
    if parts.last() == Some(&"mod") {
      parts.pop();
    }
    parts.join("::")
  };
  format!("::{module}::{function}")
}
