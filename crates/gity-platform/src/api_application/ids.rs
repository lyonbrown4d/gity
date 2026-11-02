//! Public string identifiers and schema-directed conversion at the application boundary.
use gity_api_contracts::{ApiError, ApiResult};
use serde_json::{Value, json};

pub(super) fn public_schema(schema: &mut Value) {
  match schema {
    Value::Object(object) => {
      if let Some(properties) = object.get_mut("properties").and_then(Value::as_object_mut) {
        for (name, property) in properties {
          if name == "id" || name.ends_with("_id") {
            identifier(property);
          } else if name.ends_with("_ids")
            && let Some(items) = property.get_mut("items")
          {
            identifier(items);
          }
        }
      }
      for child in object.values_mut() {
        public_schema(child);
      }
    }
    Value::Array(items) => {
      for child in items {
        public_schema(child);
      }
    }
    _ => {}
  }
}
fn identifier(schema: &mut Value) {
  let Some(object) = schema.as_object_mut() else {
    return;
  };
  let integer = object.get("type").is_some_and(|kind| {
    kind == "integer"
      || kind
        .as_array()
        .is_some_and(|types| types.contains(&json!("integer")))
  });
  if integer {
    if let Some(kind) = object.get_mut("type") {
      if let Some(types) = kind.as_array_mut() {
        for kind in types {
          if kind == "integer" {
            *kind = json!("string");
          }
        }
      } else {
        *kind = json!("string");
      }
    }
    object.insert("format".into(), json!("snowflake-id"));
    object.insert("pattern".into(), json!("^[0-9]+$"));
    object.insert("x-gity-integer".into(), json!(true));
    object.remove("minimum");
    object.remove("maximum");
  }
  for keyword in ["anyOf", "oneOf", "allOf"] {
    if let Some(parts) = object.get_mut(keyword).and_then(Value::as_array_mut) {
      for part in parts {
        identifier(part);
      }
    }
  }
}

pub(super) fn native_arguments(schema: &Value, value: &mut Value) -> ApiResult<()> {
  decode(schema, schema, value, 0)
}
fn decode(root: &Value, schema: &Value, value: &mut Value, depth: usize) -> ApiResult<()> {
  if depth > 64 {
    return Err(invalid());
  }
  if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
    let target = reference
      .strip_prefix('#')
      .and_then(|pointer| root.pointer(pointer))
      .ok_or_else(invalid)?;
    return decode(root, target, value, depth + 1);
  }
  if schema["x-gity-integer"] == true && value.is_string() {
    *value = json!(
      value
        .as_str()
        .and_then(|text| text.parse::<i64>().ok())
        .ok_or_else(invalid)?
    );
    return Ok(());
  }
  if let Some(object) = value.as_object_mut() {
    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
      for (name, property) in properties {
        if let Some(value) = object.get_mut(name) {
          decode(root, property, value, depth + 1)?;
        }
      }
    }
  } else if let Some(items) = value.as_array_mut()
    && let Some(item_schema) = schema.get("items")
  {
    for value in items {
      decode(root, item_schema, value, depth + 1)?;
    }
  }
  for keyword in ["allOf", "anyOf", "oneOf"] {
    if let Some(parts) = schema.get(keyword).and_then(Value::as_array) {
      for part in parts {
        decode(root, part, value, depth + 1)?;
      }
    }
  }
  Ok(())
}
fn invalid() -> ApiError {
  super::output::error(400, "INVALID_ARGUMENT", "invalid identifier")
}

#[cfg(test)]
mod tests;
