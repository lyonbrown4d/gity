use std::collections::{HashMap, HashSet};

use async_graphql::dynamic::{Enum, InputObject, InputValue, Scalar, Type, TypeRef};
use serde_json::Value;

use crate::{
  refs::schema_type,
  schema::{SchemaTypes, type_name},
};

#[derive(Default)]
pub(crate) struct InputTypes {
  built: HashSet<String>,
  reference_names: HashMap<String, String>,
  json_scalar: bool,
}

impl InputTypes {
  pub(crate) fn input_type(
    &mut self,
    root: &Value,
    schema: &Value,
    name: &str,
    types: &mut Vec<Type>,
  ) -> Result<TypeRef, String> {
    let reference = schema
      .get("$ref")
      .and_then(Value::as_str)
      .map(str::to_owned);
    if let Some(reference) = reference.as_deref()
      && let Some(name) = self.reference_names.get(reference)
    {
      return Ok(TypeRef::named(name.clone()));
    }
    let schema = schema_type(root, schema)?;
    let oneof = schema
      .get("__gity_oneof")
      .and_then(Value::as_bool)
      .unwrap_or(false);
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
      let choices = values
        .iter()
        .filter(|value| !value.is_null())
        .collect::<Vec<_>>();
      if !choices.is_empty()
        && choices
          .iter()
          .all(|v| v.as_str().is_some_and(crate::schema::valid_name))
      {
        let enum_name = type_name(name);
        if self.built.insert(enum_name.clone()) {
          let items = choices
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>();
          types.push(
            Enum::new(enum_name.clone())
              .items(items.iter().copied())
              .into(),
          );
        }
        return Ok(TypeRef::named(enum_name));
      }
    }
    if schema.get("type").and_then(Value::as_str) == Some("array") {
      let item = schema.get("items").ok_or("array schema is missing items")?;
      let item_type = self.input_type(root, item, &format!("{name}Item"), types)?;
      return Ok(TypeRef::List(Box::new(item_type)));
    }
    match schema.get("type").and_then(Value::as_str) {
      Some("object") => {
        let properties = schema.get("properties").and_then(Value::as_object);
        if properties.is_none_or(|map| map.is_empty()) {
          if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
            return Err("closed empty input objects have no GraphQL representation".into());
          }
          if !self.json_scalar {
            types.push(Scalar::new("JSON").validator(|_| true).into());
            self.json_scalar = true;
          }
          return Ok(TypeRef::named("JSON"));
        }
        let object_name = type_name(name);
        if !self.built.insert(object_name.clone()) {
          return Ok(TypeRef::named(object_name));
        }
        if let Some(reference) = reference.as_ref() {
          self
            .reference_names
            .insert(reference.clone(), object_name.clone());
        }
        let required = schema.get("required").and_then(Value::as_array);
        let mut object = InputObject::new(object_name.clone());
        for (property, property_schema) in properties.expect("checked above") {
          if !crate::schema::valid_name(property) {
            return Err(format!("invalid GraphQL input property: {property}"));
          }
          let child_name = format!("{object_name}{}", type_name(property));
          let mut ty = self.input_type(root, property_schema, &child_name, types)?;
          if !oneof
            && required
              .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(property)))
            && !crate::refs::accepts_null(root, property_schema)?
          {
            ty = TypeRef::NonNull(Box::new(ty));
          }
          object = object.field(InputValue::new(property, ty));
        }
        if oneof {
          object = object.oneof();
        }
        types.push(object.into());
        if let Some(reference) = reference {
          self.reference_names.remove(&reference);
        }
        Ok(TypeRef::named(object_name))
      }
      Some("string") if is_id_schema(&schema) => Ok(TypeRef::named(TypeRef::ID)),
      Some("string") => Ok(TypeRef::named(TypeRef::STRING)),
      Some("integer") if schema.get("format").and_then(Value::as_str) == Some("int64") => {
        Ok(TypeRef::named(TypeRef::ID))
      }
      Some("integer") => Ok(TypeRef::named(TypeRef::INT)),
      Some("number") => Ok(TypeRef::named(TypeRef::FLOAT)),
      Some("boolean") => Ok(TypeRef::named(TypeRef::BOOLEAN)),
      None if schema.get("$ref").is_some() => unreachable!("resolved refs above"),
      Some(kind) => Err(format!(
        "unsupported GraphQL input JSON Schema type: {kind}"
      )),
      None => {
        if !self.json_scalar {
          types.push(Scalar::new("JSON").validator(|_| true).into());
          self.json_scalar = true;
        }
        Ok(TypeRef::named("JSON"))
      }
    }
  }
}

fn is_id_schema(schema: &Value) -> bool {
  matches!(
    schema.get("format").and_then(Value::as_str),
    Some("id" | "int64" | "snowflake-id")
  )
}

pub(crate) fn normalize_input(root: &Value, schema: &Value, value: Value) -> Result<Value, String> {
  if value.is_null() && crate::refs::accepts_null(root, schema)? {
    return Ok(Value::Null);
  }
  let resolved = crate::refs::dereference(root, schema)?;
  if let Some(options) = resolved.get("oneOf").and_then(Value::as_array) {
    let object = value.as_object().ok_or("oneOf input must be an object")?;
    let selected = options
      .iter()
      .enumerate()
      .find_map(|(index, option)| {
        let branch = crate::refs::dereference(root, option).ok()?;
        let name = crate::refs::alternative_name(option, branch, index);
        object.get(&name).map(|value| (option, value))
      })
      .ok_or("oneOf input must select one alternative")?;
    return normalize_input(root, selected.0, selected.1.clone());
  }
  let normalized_schema = crate::refs::schema_type(root, schema)?;
  if let Some(properties) = normalized_schema
    .get("properties")
    .and_then(Value::as_object)
  {
    let Some(object) = value.as_object() else {
      return Ok(value);
    };
    let mut normalized = object.clone();
    for (name, property_schema) in properties {
      if let Some(property_value) = object.get(name) {
        normalized.insert(
          name.clone(),
          normalize_input(root, property_schema, property_value.clone())?,
        );
      }
    }
    return Ok(Value::Object(normalized));
  }
  if let (Some(items), Some(values)) = (resolved.get("items"), value.as_array()) {
    return values
      .iter()
      .map(|item| normalize_input(root, items, item.clone()))
      .collect::<Result<Vec<_>, _>>()
      .map(Value::Array);
  }
  Ok(value)
}

impl SchemaTypes {
  pub(crate) fn input_type(
    &mut self,
    root: &Value,
    schema: &Value,
    name: &str,
    types: &mut Vec<Type>,
  ) -> Result<TypeRef, String> {
    self.inputs.input_type(root, schema, name, types)
  }
}
