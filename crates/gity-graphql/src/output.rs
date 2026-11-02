use std::collections::{HashMap, HashSet};

use async_graphql::{
  Name, Value as GraphValue,
  dynamic::{Enum, Field, FieldFuture, FieldValue, Object, ResolverContext, Scalar, Type, TypeRef},
};
use serde_json::Value;

use crate::{
  refs::schema_type,
  schema::{SchemaTypes, type_name},
};

#[derive(Default)]
pub(crate) struct OutputTypes {
  built: HashSet<String>,
  reference_names: HashMap<String, String>,
  json_scalar: bool,
}

impl OutputTypes {
  pub(crate) fn output_type(
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
          types.push(
            Enum::new(enum_name.clone())
              .items(choices.iter().filter_map(|value| value.as_str()))
              .into(),
          );
        }
        return Ok(TypeRef::named(enum_name));
      }
    }
    if schema.get("type").and_then(Value::as_str) == Some("array") {
      let item = schema.get("items").ok_or("array schema is missing items")?;
      let item_type = self.output_type(root, item, &format!("{name}Item"), types)?;
      return Ok(TypeRef::List(Box::new(item_type)));
    }
    match schema.get("type").and_then(Value::as_str) {
      Some("object") => {
        let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
          if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
            return Err("closed empty output objects have no GraphQL representation".into());
          }
          return self.json_type(types);
        };
        if properties.is_empty() {
          if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
            return Err("closed empty output objects have no GraphQL representation".into());
          }
          return self.json_type(types);
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
        let mut object = Object::new(object_name.clone());
        for (property, property_schema) in properties {
          if !crate::schema::valid_name(property) {
            return Err(format!("invalid GraphQL output property: {property}"));
          }
          let child_name = format!("{object_name}{}", type_name(property));
          let mut ty = self.output_type(root, property_schema, &child_name, types)?;
          if required.is_some_and(|items| items.iter().any(|item| item.as_str() == Some(property)))
            && !crate::refs::accepts_null(root, property_schema)?
          {
            ty = TypeRef::NonNull(Box::new(ty));
          }
          let key = property.clone();
          let child_schema = property_schema.clone();
          let root_schema = root.clone();
          let graphql_type = ty.clone();
          let resolver = output_resolver(key, root_schema, child_schema, graphql_type);
          object = object.field(Field::new(property, ty, resolver));
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
      None => self.json_type(types),
      Some(kind) => Err(format!(
        "unsupported GraphQL output JSON Schema type: {kind}"
      )),
    }
  }

  fn json_type(&mut self, types: &mut Vec<Type>) -> Result<TypeRef, String> {
    if !self.json_scalar {
      types.push(Scalar::new("JSON").validator(|_| true).into());
      self.json_scalar = true;
    }
    Ok(TypeRef::named("JSON"))
  }
}

fn is_id_schema(schema: &Value) -> bool {
  matches!(
    schema.get("format").and_then(Value::as_str),
    Some("id" | "int64" | "snowflake-id")
  )
}

fn output_resolver(
  key: String,
  root: Value,
  schema: Value,
  graphql_type: TypeRef,
) -> impl for<'a> Fn(ResolverContext<'a>) -> FieldFuture<'a> + Send + Sync + 'static {
  move |ctx| {
    FieldFuture::new(resolve_output_field(
      ctx,
      key.clone(),
      root.clone(),
      schema.clone(),
      graphql_type.clone(),
    ))
  }
}

async fn resolve_output_field<'a>(
  ctx: ResolverContext<'a>,
  key: String,
  root: Value,
  schema: Value,
  graphql_type: TypeRef,
) -> Result<Option<FieldValue<'a>>, async_graphql::Error> {
  let parent = ctx.parent_value.try_downcast_ref::<Value>()?;
  let value = parent.get(&key).cloned().unwrap_or(Value::Null);
  json_field_value(value, &root, &schema, &graphql_type)
}

impl SchemaTypes {
  pub(crate) fn output_type(
    &mut self,
    root: &Value,
    schema: &Value,
    name: &str,
    _operation: &str,
    types: &mut Vec<Type>,
  ) -> Result<TypeRef, String> {
    let mut output = std::mem::take(&mut self.outputs);
    let result = output.output_type(root, schema, name, types);
    self.outputs = output;
    result
  }
}

pub(crate) fn json_field_value<'a>(
  value: Value,
  root: &Value,
  schema: &Value,
  graphql_type: &TypeRef,
) -> Result<Option<FieldValue<'a>>, async_graphql::Error> {
  if value.is_null() {
    return Ok(Some(FieldValue::NULL));
  }
  if let Some(union) = crate::refs::dereference(root, schema)
    .map_err(async_graphql::Error::new)?
    .get("oneOf")
    .and_then(Value::as_array)
  {
    let matching = union
      .iter()
      .enumerate()
      .filter_map(|(index, option)| {
        let branch = crate::refs::dereference(root, option).ok()?;
        crate::refs::branch_matches(branch, &value)
          .then(|| crate::refs::alternative_name(option, branch, index))
      })
      .collect::<Vec<_>>();
    if matching.len() != 1 {
      return Err(async_graphql::Error::new(
        "backend returned an ambiguous or invalid oneOf value",
      ));
    }
    let variant = &matching[0];
    return Ok(Some(FieldValue::owned_any(
      serde_json::json!({variant: value}),
    )));
  }
  let schema = schema_type(root, schema).map_err(async_graphql::Error::new)?;
  if schema.get("type").and_then(Value::as_str) == Some("array") {
    let Some(items) = value.as_array() else {
      return Err(async_graphql::Error::new(
        "backend returned an invalid array value",
      ));
    };
    let item_schema = schema.get("items").cloned().unwrap_or(Value::Null);
    let item_type = match graphql_type {
      TypeRef::List(inner) => inner.as_ref(),
      _ => graphql_type,
    };
    let converted = items
      .iter()
      .map(|item| json_field_value(item.clone(), root, &item_schema, item_type))
      .collect::<Result<Vec<_>, _>>()?;
    return Ok(Some(FieldValue::list(converted.into_iter().flatten())));
  }
  if schema.get("type").and_then(Value::as_str) == Some("object") {
    if graphql_type.type_name() == "JSON" {
      return Ok(Some(FieldValue::value(
        GraphValue::from_json(value).map_err(|e| async_graphql::Error::new(e.to_string()))?,
      )));
    }
    return Ok(Some(FieldValue::owned_any(value)));
  }
  if schema.get("enum").is_some() {
    let value = value
      .as_str()
      .ok_or_else(|| async_graphql::Error::new("backend returned an invalid enum value"))?;
    return Ok(Some(FieldValue::value(GraphValue::Enum(Name::new(value)))));
  }
  if (schema.get("type").and_then(Value::as_str) == Some("integer")
    && schema.get("format").and_then(Value::as_str) == Some("int64"))
    || (schema.get("type").and_then(Value::as_str) == Some("string") && is_id_schema(&schema))
  {
    let id = value
      .as_i64()
      .map(|number| number.to_string())
      .or_else(|| value.as_str().map(str::to_owned))
      .ok_or_else(|| async_graphql::Error::new("backend returned an invalid ID value"))?;
    return Ok(Some(FieldValue::value(GraphValue::String(id))));
  }
  Ok(Some(FieldValue::value(
    GraphValue::from_json(value).map_err(|e| async_graphql::Error::new(e.to_string()))?,
  )))
}
