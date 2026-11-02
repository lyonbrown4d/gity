use serde_json::{Value, json};

pub(crate) fn is_array_output(schema: &Value) -> bool {
  local_dereference(schema).and_then(|resolved| resolved.get("type").and_then(Value::as_str))
    == Some("array")
}

pub(super) fn pagination_schema() -> Value {
  json!({
    "type": "object",
    "properties": {
      "page": {
        "type": "integer",
        "minimum": 1,
        "maximum": 1000000,
        "default": 1,
        "description": "One-based result page."
      },
      "page_size": {
        "type": "integer",
        "minimum": 1,
        "maximum": 100,
        "default": 50,
        "description": "Maximum number of returned entries."
      }
    },
    "additionalProperties": false
  })
}

fn local_dereference(mut schema: &Value) -> Option<&Value> {
  let mut seen = std::collections::HashSet::new();
  while let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
    if !seen.insert(reference) {
      return None;
    }
    schema = schema.pointer(reference.strip_prefix('#')?)?;
  }
  Some(schema)
}
