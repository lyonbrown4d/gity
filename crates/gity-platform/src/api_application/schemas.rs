use serde_json::{Map, Value, json};
mod pagination;
mod response;
pub(super) use pagination::is_array_output;
use pagination::pagination_schema;
pub(super) use response::output_schema;

pub(super) fn input_schema(
  document: &Value,
  parameters: &[Value],
  request_body: Option<&Value>,
  pagination: bool,
) -> Result<(Value, Vec<(String, bool)>), String> {
  let mut definitions = Map::new();
  let mut path_properties = Map::new();
  let mut query_properties = Map::new();
  let mut path_required = Vec::new();
  let mut query_required = Vec::new();
  let mut path_order = Vec::new();

  let mut seen = std::collections::HashSet::new();
  for parameter in parameters {
    let name = parameter
      .get("name")
      .and_then(Value::as_str)
      .ok_or("OpenAPI parameter has no name")?;
    let location = parameter
      .get("in")
      .and_then(Value::as_str)
      .ok_or_else(|| format!("OpenAPI parameter {name} has no location"))?;
    if !seen.insert((location, name)) {
      continue;
    }
    let source = parameter
      .get("schema")
      .or_else(|| parameter.get("content").and_then(first_content_schema))
      .ok_or_else(|| format!("OpenAPI parameter {name} has no schema"))?;
    let integer = is_integer(document, source)?;
    let mut property = rewrite_schema(document, source, &mut definitions)?;
    if location == "path" {
      path_order.push((name.to_owned(), integer));
      if integer {
        property = public_id_schema(property);
      }
      path_properties.insert(name.to_owned(), property);
      if parameter.get("required").and_then(Value::as_bool) != Some(false) {
        path_required.push(Value::String(name.to_owned()));
      }
    } else if location == "query" {
      query_properties.insert(name.to_owned(), property);
      if parameter.get("required").and_then(Value::as_bool) == Some(true) {
        query_required.push(Value::String(name.to_owned()));
      }
    }
  }

  let mut path = object_schema(path_properties);
  if !path_required.is_empty() {
    path["required"] = Value::Array(path_required);
  }
  let has_required_query = !query_required.is_empty();
  let mut query = object_schema(query_properties);
  if !query_required.is_empty() {
    query["required"] = Value::Array(query_required);
  }
  let mut properties = Map::new();
  let mut required = Vec::new();
  let has_path = path["properties"]
    .as_object()
    .is_some_and(|properties| !properties.is_empty());
  if has_path {
    properties.insert("path".into(), path);
  }
  if query["properties"]
    .as_object()
    .is_some_and(|properties| !properties.is_empty())
  {
    properties.insert("query".into(), query);
  }
  if has_path {
    required.push(Value::String("path".into()));
  }
  if has_required_query {
    required.push(Value::String("query".into()));
  }

  if let Some(body) = request_body {
    let body_schema = body
      .get("content")
      .and_then(first_content_schema)
      .ok_or("OpenAPI request body has no JSON schema")?;
    let body_schema = rewrite_schema(document, body_schema, &mut definitions)?;
    properties.insert("body".into(), body_schema);
    if body.get("required").and_then(Value::as_bool) == Some(true) {
      required.push(Value::String("body".into()));
    }
  }
  if pagination {
    properties.insert("pagination".into(), pagination_schema());
  }

  let mut input = object_schema(properties);
  input["required"] = Value::Array(required);
  if !definitions.is_empty() {
    input["$defs"] = Value::Object(definitions);
  }
  Ok((input, path_order))
}

pub(super) fn first_content_schema(content: &Value) -> Option<&Value> {
  content
    .as_object()?
    .values()
    .find_map(|media| media.get("schema"))
}

fn object_schema(properties: Map<String, Value>) -> Value {
  json!({
    "type": "object",
    "properties": properties,
    "additionalProperties": false
  })
}

fn is_integer(document: &Value, schema: &Value) -> Result<bool, String> {
  let schema = dereference(document, schema)?;
  Ok(
    schema
      .get("type")
      .and_then(Value::as_str)
      .is_some_and(|kind| kind == "integer"),
  )
}

fn public_id_schema(schema: Value) -> Value {
  let mut output = json!({ "type": "string", "format": "snowflake-id" });
  if let (Some(source), Some(target)) = (schema.as_object(), output.as_object_mut()) {
    for key in ["title", "description", "example", "default"] {
      if let Some(value) = source.get(key) {
        target.insert(key.into(), value.clone());
      }
    }
  }
  output
}

pub(super) fn dereference<'a>(
  document: &'a Value,
  mut schema: &'a Value,
) -> Result<&'a Value, String> {
  let mut seen = std::collections::HashSet::new();
  while let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
    if !seen.insert(reference) {
      return Err(format!("cyclic OpenAPI schema reference: {reference}"));
    }
    let pointer = reference
      .strip_prefix("#")
      .ok_or_else(|| format!("external OpenAPI schema reference is unsupported: {reference}"))?;
    schema = document
      .pointer(pointer)
      .ok_or_else(|| format!("unresolved OpenAPI schema reference: {reference}"))?;
  }
  Ok(schema)
}

pub(super) fn rewrite_schema(
  document: &Value,
  schema: &Value,
  definitions: &mut Map<String, Value>,
) -> Result<Value, String> {
  if !schema.is_object() {
    return Ok(schema.clone());
  }
  if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
    let name = reference
      .strip_prefix("#/components/schemas/")
      .ok_or_else(|| format!("unsupported OpenAPI schema reference: {reference}"))?;
    let definition = document
      .pointer(&format!("/components/schemas/{}", escape_pointer(name)))
      .ok_or_else(|| format!("unresolved OpenAPI schema reference: {reference}"))?;
    if !definitions.contains_key(name) {
      definitions.insert(name.to_owned(), Value::Null);
      let converted = rewrite_schema(document, definition, definitions)?;
      definitions.insert(name.to_owned(), converted);
    }
    let mut result = schema.clone();
    result["$ref"] = Value::String(format!("#/$defs/{name}"));
    return Ok(result);
  }

  if let Some(parts) = schema.get("allOf").and_then(Value::as_array) {
    let mut merged = Map::new();
    for part in parts {
      let expanded = if part.get("$ref").is_some() {
        dereference(document, part)?.clone()
      } else {
        part.clone()
      };
      let expanded = rewrite_schema(document, &expanded, definitions)?;
      merge_all_of(
        &mut merged,
        expanded
          .as_object()
          .ok_or("allOf member must be an object")?,
      )?;
    }
    for (key, value) in schema.as_object().ok_or("schema must be an object")? {
      if key != "allOf" {
        merged.insert(key.clone(), rewrite_schema(document, value, definitions)?);
      }
    }
    return Ok(Value::Object(merged));
  }

  let mut output = Map::new();
  for (key, value) in schema.as_object().ok_or("schema must be an object")? {
    output.insert(
      key.clone(),
      match value {
        Value::Object(_) => rewrite_schema(document, value, definitions)?,
        Value::Array(items) => Value::Array(
          items
            .iter()
            .map(|item| {
              if item.is_object() {
                rewrite_schema(document, item, definitions)
              } else {
                Ok(item.clone())
              }
            })
            .collect::<Result<Vec<_>, String>>()?,
        ),
        _ => value.clone(),
      },
    );
  }
  if output.get("nullable").and_then(Value::as_bool) == Some(true) {
    output.remove("nullable");
    if let Some(kind) = output
      .get("type")
      .and_then(Value::as_str)
      .map(str::to_owned)
    {
      output.insert("type".into(), json!([kind, "null"]));
    }
  }
  Ok(Value::Object(output))
}

fn merge_all_of(
  target: &mut Map<String, Value>,
  source: &Map<String, Value>,
) -> Result<(), String> {
  for (key, value) in source {
    if key == "properties" {
      let source = value
        .as_object()
        .ok_or("allOf properties must be an object")?;
      let target = target
        .entry(key.clone())
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or("allOf properties conflict with another schema type")?;
      target.extend(source.clone());
    } else if key == "required" {
      let source = value.as_array().ok_or("allOf required must be an array")?;
      let target = target
        .entry(key.clone())
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or("allOf required conflicts with another schema type")?;
      for item in source {
        if !target.contains(item) {
          target.push(item.clone());
        }
      }
    } else if let Some(existing) = target.get(key) {
      if existing != value {
        return Err(format!("conflicting allOf keyword: {key}"));
      }
    } else {
      target.insert(key.clone(), value.clone());
    }
  }
  Ok(())
}

fn escape_pointer(value: &str) -> String {
  value.replace('~', "~0").replace('/', "~1")
}
