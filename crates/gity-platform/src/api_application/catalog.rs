use super::{OperationRegistration, schemas};
use gity_api_contracts::{OperationContract, OperationKind};
use serde_json::Value;
use std::collections::HashSet;

pub(crate) struct CatalogOperation {
  pub contract: OperationContract,
  pub registration: &'static OperationRegistration,
  pub path_parameters: Vec<(String, bool)>,
}

pub(crate) fn build() -> Result<Vec<CatalogOperation>, String> {
  let document = serde_json::to_value(crate::openapi::document())
    .map_err(|error| format!("cannot serialize OpenAPI operation document: {error}"))?;
  let mut catalog = Vec::new();
  let mut names = HashSet::new();
  let mut routes = HashSet::new();

  let mut registrations: Vec<_> = inventory::iter::<OperationRegistration>
    .into_iter()
    .collect();
  registrations.sort_by_key(|entry| (entry.prefix, entry.path, entry.method, entry.handler));
  for registration in registrations {
    let path = join_path(registration.prefix, registration.path);
    let key = (registration.method.to_ascii_lowercase(), path.clone());
    if !routes.insert(key.clone()) {
      return Err(format!("duplicate application route: {} {}", key.0, key.1));
    }
    let operation = documented_operation(&document, &key.0, &path)?;
    let parameters = operation
      .get("parameters")
      .and_then(Value::as_array)
      .map(Vec::as_slice)
      .unwrap_or_default();
    let request_body = operation.get("requestBody");
    let output_schema = schemas::output_schema(&document, operation)
      .map_err(|error| format!("{} {path} output schema: {error}", key.0))?;
    let pagination = schemas::is_array_output(&output_schema);
    let (mut input_schema, mut path_parameters) =
      schemas::input_schema(&document, parameters, request_body, pagination)
        .map_err(|error| format!("{} {path} input schema: {error}", key.0))?;
    let positions: Vec<_> = path
      .split('{')
      .skip(1)
      .filter_map(|part| part.split('}').next())
      .collect();
    path_parameters.sort_by_key(|(name, _)| {
      positions
        .iter()
        .position(|position| *position == name)
        .unwrap_or(usize::MAX)
    });
    super::ids::public_schema(&mut input_schema);
    let group = operation_group(operation, registration);
    let name = operation_name(&group, registration, &path, &names);
    if !names.insert(name.clone()) {
      return Err(format!("duplicate application operation name: {name}"));
    }
    let kind = if key.0 == "get" || key.0 == "head" || (key.0 == "post" && path == "/api/v1/search")
    {
      OperationKind::Query
    } else {
      OperationKind::Mutation
    };
    let contract = OperationContract {
      name,
      description: operation_description(operation, registration, &path, pagination),
      group,
      kind,
      input_schema,
      output_schema,
      destructive: is_destructive(registration, &key.0, &path),
      idempotent: is_idempotent(&key.0),
    };
    catalog.push(CatalogOperation {
      contract,
      registration,
      path_parameters,
    });
  }
  if catalog.is_empty() {
    return Err("no shared application operations were registered".into());
  }
  catalog.sort_by(|left, right| left.contract.name.cmp(&right.contract.name));
  Ok(catalog)
}

fn documented_operation<'a>(
  document: &'a Value,
  method: &str,
  path: &str,
) -> Result<&'a Value, String> {
  document
    .pointer(&format!("/paths/{}", escape_pointer(path)))
    .and_then(|item| item.get(method))
    .ok_or_else(|| format!("registered operation {method} {path} is absent from OpenAPI"))
}

fn join_path(prefix: &str, path: &str) -> String {
  let prefix = prefix.trim_matches('/');
  let path = path.trim_matches('/');
  match (prefix.is_empty(), path.is_empty()) {
    (true, true) => "/".into(),
    (true, false) => format!("/{path}"),
    (false, true) => format!("/{prefix}"),
    (false, false) => format!("/{prefix}/{path}"),
  }
}

fn operation_group(operation: &Value, registration: &OperationRegistration) -> String {
  operation
    .get("tags")
    .and_then(Value::as_array)
    .and_then(|tags| tags.first())
    .and_then(Value::as_str)
    .map(snake_case)
    .filter(|group| !group.is_empty())
    .unwrap_or_else(|| {
      let parts = registration.handler.split("::").collect::<Vec<_>>();
      parts
        .get(parts.len().saturating_sub(2))
        .map(|value| snake_case(value))
        .filter(|group| !group.is_empty())
        .unwrap_or_else(|| "application".into())
    })
}

fn operation_name(
  group: &str,
  registration: &OperationRegistration,
  path: &str,
  existing: &HashSet<String>,
) -> String {
  let parts = registration.handler.split("::").collect::<Vec<_>>();
  let handler = registration
    .handler
    .rsplit("::")
    .next()
    .unwrap_or("operation");
  let group = snake_case(group);
  let module = parts
    .get(parts.len().saturating_sub(2))
    .map(|part| snake_case(part))
    .unwrap_or_else(|| "application".into());
  let handler = snake_case(handler);
  let name = if module == group || module == "mod" {
    format!("{group}_{handler}")
  } else {
    format!("{group}_{module}_{handler}")
  };
  let name = if existing.contains(&name) {
    format!("{name}_{}", snake_case(path))
  } else {
    name
  };
  if name
    .chars()
    .next()
    .is_some_and(|character| character.is_ascii_digit())
  {
    format!("operation_{name}")
  } else {
    name
  }
}

fn description(operation: &Value, registration: &OperationRegistration, path: &str) -> String {
  operation
    .get("summary")
    .and_then(Value::as_str)
    .filter(|summary| !summary.trim().is_empty())
    .or_else(|| operation.get("description").and_then(Value::as_str))
    .map(str::to_owned)
    .unwrap_or_else(|| format!("{} operation for {path}", registration.handler))
}

fn operation_description(
  operation: &Value,
  registration: &OperationRegistration,
  path: &str,
  pagination: bool,
) -> String {
  let description = description(operation, registration, path);
  if pagination {
    format!(
      "{description} Result arrays accept pagination.page (default 1) and pagination.page_size (default 50, maximum 100)."
    )
  } else {
    description
  }
}

fn is_idempotent(method: &str) -> bool {
  matches!(method, "get" | "head" | "put" | "delete")
}

fn is_destructive(registration: &OperationRegistration, method: &str, path: &str) -> bool {
  if method == "delete" {
    return true;
  }
  let action = registration
    .handler
    .rsplit("::")
    .next()
    .unwrap_or_default()
    .to_ascii_lowercase();
  let path = path.to_ascii_lowercase();
  [
    "abort",
    "cancel",
    "delete",
    "remove",
    "revoke",
    "unapprove",
    "unprotect",
    "unlink",
    "merge",
  ]
  .iter()
  .any(|verb| action.contains(verb) || path.contains(&format!("/{verb}")))
}

fn snake_case(value: &str) -> String {
  let mut result = String::new();
  let mut previous_lowercase = false;
  for character in value.chars() {
    if character.is_ascii_alphanumeric() {
      if character.is_ascii_uppercase() && previous_lowercase && !result.ends_with('_') {
        result.push('_');
      }
      result.push(character.to_ascii_lowercase());
      previous_lowercase = character.is_ascii_lowercase() || character.is_ascii_digit();
    } else if !result.is_empty() && !result.ends_with('_') {
      result.push('_');
      previous_lowercase = false;
    }
  }
  result.trim_matches('_').to_owned()
}

fn escape_pointer(value: &str) -> String {
  value.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests;
