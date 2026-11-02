use super::{dereference, first_content_schema, rewrite_schema};
use serde_json::{Map, Value};

pub(crate) fn output_schema(document: &Value, operation: &Value) -> Result<Value, String> {
  let responses = operation
    .get("responses")
    .and_then(Value::as_object)
    .ok_or("OpenAPI operation has no responses")?;
  let (status, response) = responses
    .iter()
    .filter(|(status, _)| status.starts_with('2'))
    .min_by_key(|(status, _)| status.parse::<u16>().unwrap_or(299))
    .ok_or("OpenAPI operation has no successful response")?;
  let Some(body_schema) = response.get("content").and_then(first_content_schema) else {
    if status == "204" {
      return Ok(serde_json::json!({
        "type": "object",
        "properties": { "ok": { "type": "boolean" } },
        "required": ["ok"],
        "additionalProperties": false
      }));
    }
    return Err("OpenAPI success response has no JSON schema".into());
  };
  let data_schema = envelope_data_schema(document, body_schema)?;
  let mut definitions = Map::new();
  let output = rewrite_schema(document, data_schema, &mut definitions)?;
  if definitions.is_empty() {
    Ok(output)
  } else {
    let mut output = output;
    output["$defs"] = Value::Object(definitions);
    Ok(output)
  }
}

fn envelope_data_schema<'a>(document: &'a Value, schema: &'a Value) -> Result<&'a Value, String> {
  let resolved = dereference(document, schema)?;
  if let Some(data) = resolved
    .get("properties")
    .and_then(|properties| properties.get("data"))
  {
    return Ok(data);
  }
  if let Some(parts) = resolved.get("allOf").and_then(Value::as_array) {
    for part in parts {
      if let Ok(data) = envelope_data_schema(document, part) {
        return Ok(data);
      }
    }
  }
  Ok(schema)
}
