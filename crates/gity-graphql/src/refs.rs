use serde_json::Value;

pub(crate) fn dereference<'a>(root: &'a Value, schema: &'a Value) -> Result<&'a Value, String> {
  let mut current = schema;
  let mut visited = std::collections::HashSet::new();
  loop {
    let Some(reference) = current.get("$ref").and_then(Value::as_str) else {
      return Ok(current);
    };
    if !visited.insert(reference) {
      return Err(format!("cyclic JSON Schema alias reference: {reference}"));
    }
    let Some(pointer) = reference.strip_prefix('#') else {
      return Err(format!(
        "external JSON Schema reference is unsupported: {reference}"
      ));
    };
    current = root
      .pointer(pointer)
      .ok_or_else(|| format!("unresolved JSON Schema reference: {reference}"))?;
  }
}

pub(crate) fn schema_type(root: &Value, schema: &Value) -> Result<Value, String> {
  let resolved = dereference(root, schema)?;
  if !resolved.is_object() {
    return Err("boolean JSON Schema values cannot be represented by GraphQL".into());
  }
  if let Some(parts) = resolved.get("allOf") {
    let parts = parts.as_array().ok_or("allOf must be an array")?;
    let mut merged = serde_json::Map::new();
    for part in parts {
      let normalized = schema_type(root, part)?;
      merge_schema(
        &mut merged,
        normalized
          .as_object()
          .ok_or("allOf members must be objects")?,
      )?;
    }
    merge_schema(
      &mut merged,
      resolved.as_object().ok_or("schema must be an object")?,
    )?;
    let merged = Value::Object(merged);
    return schema_type(root, &merged);
  }
  let is_one_of = resolved.get("oneOf").is_some();
  let nullable = schema_may_be_null(root, schema) || schema_may_be_null(root, resolved);
  if let Some(union) = resolved.get("oneOf").or_else(|| resolved.get("anyOf")) {
    let options = union.as_array().ok_or("oneOf/anyOf must be an array")?;
    let mut properties = serde_json::Map::new();
    for (index, option) in options.iter().enumerate() {
      let item = dereference(root, option)?;
      if item.get("type").and_then(Value::as_str) == Some("null") {
        continue;
      }
      if is_one_of {
        properties.insert(alternative_name(option, item, index), option.clone());
        continue;
      }
      if let Some(fields) = item.get("properties").and_then(Value::as_object) {
        for (key, value) in fields {
          properties
            .entry(key.clone())
            .or_insert_with(|| value.clone());
        }
      } else if item.get("type").and_then(Value::as_str) != Some("object") {
        return Err("oneOf/anyOf currently require object alternatives".into());
      }
    }
    let mut result = serde_json::Map::new();
    result.insert("type".into(), Value::String("object".into()));
    result.insert("properties".into(), Value::Object(properties));
    if nullable {
      result.insert("__gity_nullable".into(), Value::Bool(true));
    }
    if is_one_of {
      result.insert("__gity_oneof".into(), Value::Bool(true));
    }
    return Ok(Value::Object(result));
  }
  let mut normalized = resolved.clone();
  if let Some(types) = normalized.get("type").and_then(Value::as_array) {
    let non_null = types
      .iter()
      .filter_map(Value::as_str)
      .filter(|kind| *kind != "null")
      .map(str::to_owned)
      .collect::<Vec<_>>();
    if non_null.len() == 1 {
      if let Some(object) = normalized.as_object_mut() {
        object.insert("type".into(), Value::String(non_null[0].clone()));
      }
    } else if non_null.len() > 1 {
      return Err("JSON Schema type unions require one concrete type and null".into());
    }
  }
  if nullable {
    normalized
      .as_object_mut()
      .expect("resolved schema is an object")
      .insert("__gity_nullable".into(), Value::Bool(true));
  }
  if is_one_of {
    normalized
      .as_object_mut()
      .expect("oneOf is represented as object schema")
      .insert("__gity_oneof".into(), Value::Bool(true));
  }
  Ok(normalized)
}

fn merge_schema(
  target: &mut serde_json::Map<String, Value>,
  source: &serde_json::Map<String, Value>,
) -> Result<(), String> {
  for (key, value) in source {
    if key.starts_with("__gity_") || key == "allOf" {
      continue;
    }
    if key == "properties" {
      let source_properties = value
        .as_object()
        .ok_or("schema properties must be an object")?;
      let properties = target
        .entry(key.clone())
        .or_insert_with(|| Value::Object(serde_json::Map::new()))
        .as_object_mut()
        .ok_or("allOf properties conflict with a non-object schema")?;
      for (name, property) in source_properties {
        if let Some(existing) = properties.get(name) {
          if existing != property {
            return Err(format!("conflicting allOf schema for property {name}"));
          }
        } else {
          properties.insert(name.clone(), property.clone());
        }
      }
      continue;
    }
    if key == "required" {
      let source_required = value.as_array().ok_or("schema required must be an array")?;
      let required = target
        .entry(key.clone())
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or("allOf required conflicts with a non-array schema")?;
      for name in source_required {
        if !required.contains(name) {
          required.push(name.clone());
        }
      }
      continue;
    }
    if let Some(existing) = target.get(key) {
      if existing != value {
        return Err(format!("conflicting allOf schema keyword: {key}"));
      }
    } else {
      target.insert(key.clone(), value.clone());
    }
  }
  Ok(())
}

pub(crate) fn accepts_null(root: &Value, schema: &Value) -> Result<bool, String> {
  Ok(
    schema_type(root, schema)?
      .get("__gity_nullable")
      .and_then(Value::as_bool)
      .unwrap_or(false),
  )
}

fn schema_may_be_null(root: &Value, schema: &Value) -> bool {
  if schema.get("nullable").and_then(Value::as_bool) == Some(true) {
    return true;
  }
  let schema = dereference(root, schema).unwrap_or(schema);
  if schema.get("nullable").and_then(Value::as_bool) == Some(true) {
    return true;
  }
  if schema
    .get("type")
    .and_then(Value::as_array)
    .is_some_and(|types| types.iter().any(|kind| kind.as_str() == Some("null")))
  {
    return true;
  }
  if let Some(values) = schema.get("enum").and_then(Value::as_array) {
    return values.iter().any(Value::is_null);
  }
  if let Some(value) = schema.get("const") {
    return value.is_null();
  }
  if let Some(options) = schema
    .get("oneOf")
    .or_else(|| schema.get("anyOf"))
    .and_then(Value::as_array)
  {
    return options
      .iter()
      .any(|option| schema_may_be_null(root, option));
  }
  schema.get("type").is_none()
}

pub(crate) fn alternative_name(option: &Value, resolved: &Value, index: usize) -> String {
  let candidate = option
    .get("$ref")
    .and_then(Value::as_str)
    .and_then(|reference| reference.rsplit('/').next())
    .or_else(|| resolved.get("title").and_then(Value::as_str));
  let candidate = candidate
    .map(|name| name.replace("~1", "_").replace("~0", "_"))
    .unwrap_or_else(|| format!("variant{}", index + 1));
  let name = crate::schema::type_name(&candidate);
  let mut chars = name.chars();
  match chars.next() {
    Some(first) => format!("{}{}", first.to_lowercase(), chars.as_str()),
    None => format!("variant{}", index + 1),
  }
}

pub(crate) fn branch_matches(schema: &Value, value: &Value) -> bool {
  if let Some(required) = schema.get("required").and_then(Value::as_array)
    && !required
      .iter()
      .filter_map(Value::as_str)
      .all(|key| value.get(key).is_some())
  {
    return false;
  }
  if let (Some(properties), Some(object)) = (
    schema.get("properties").and_then(Value::as_object),
    value.as_object(),
  ) {
    if !properties.is_empty() && !properties.keys().any(|key| object.contains_key(key)) {
      return false;
    }
    for (key, property_schema) in properties {
      if let (Some(property_value), Some(values)) = (
        object.get(key),
        property_schema.get("enum").and_then(Value::as_array),
      ) && !values.contains(property_value)
      {
        return false;
      }
    }
  }
  match schema.get("type").and_then(Value::as_str) {
    Some("object") => value.is_object(),
    Some("array") => value.is_array(),
    Some("string") => value.is_string(),
    Some("integer") => value.as_i64().is_some(),
    Some("number") => value.is_number(),
    Some("boolean") => value.is_boolean(),
    _ => true,
  }
}
