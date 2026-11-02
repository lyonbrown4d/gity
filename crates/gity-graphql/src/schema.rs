use std::collections::HashSet;

use async_graphql::dynamic::{Schema, SchemaError, Type};
use gity_api_contracts::{OperationContract, OperationKind};
use serde_json::Value;

use crate::{input::InputTypes, output::OutputTypes, refs::schema_type};

pub(crate) fn build_schema(
  operations: &[OperationContract],
  max_depth: usize,
  max_complexity: usize,
  introspection: bool,
) -> Result<Schema, String> {
  if !operations.iter().any(|op| op.kind == OperationKind::Query) {
    return Err("GraphQL requires at least one query operation".into());
  }
  let mut names = HashSet::new();
  for operation in operations {
    if !valid_name(&operation.name) || !names.insert(operation.name.clone()) {
      return Err(format!(
        "invalid or duplicate GraphQL operation name: {}",
        operation.name
      ));
    }
  }
  let mut builder = SchemaTypes::new();
  let schema = builder.build(operations, max_depth, max_complexity, introspection)?;
  Ok(schema)
}

pub(crate) struct SchemaTypes {
  pub types: Vec<Type>,
  pub inputs: InputTypes,
  pub outputs: OutputTypes,
}

impl SchemaTypes {
  fn new() -> Self {
    Self {
      types: Vec::new(),
      inputs: InputTypes::default(),
      outputs: OutputTypes::default(),
    }
  }

  fn build(
    &mut self,
    operations: &[OperationContract],
    depth: usize,
    complexity: usize,
    introspection: bool,
  ) -> Result<Schema, String> {
    let mut query = async_graphql::dynamic::Object::new("Query");
    let mut mutation = async_graphql::dynamic::Object::new("Mutation");
    let mut has_mutation = false;
    for op in operations {
      let (field, types) = self.operation(op)?;
      self.types.extend(types);
      match op.kind {
        OperationKind::Query => query = query.field(field),
        OperationKind::Mutation => {
          mutation = mutation.field(field);
          has_mutation = true;
        }
      }
    }
    let mut schema = Schema::build("Query", has_mutation.then_some("Mutation"), None)
      .register(query)
      .limit_depth(depth)
      .limit_complexity(complexity)
      .limit_recursive_depth(depth.min(32));
    if has_mutation {
      schema = schema.register(mutation);
    }
    for ty in self.types.drain(..) {
      schema = schema.register(ty);
    }
    if !introspection {
      schema = schema.disable_introspection();
    }
    schema
      .finish()
      .map_err(|error: SchemaError| error.to_string())
  }

  fn operation(
    &mut self,
    op: &OperationContract,
  ) -> Result<(async_graphql::dynamic::Field, Vec<Type>), String> {
    let mut types = Vec::new();
    let input = schema_type(&op.input_schema, &op.input_schema)?;
    let properties = input
      .get("properties")
      .and_then(Value::as_object)
      .filter(|properties| !properties.is_empty());
    let oneof_input = input
      .get("__gity_oneof")
      .and_then(Value::as_bool)
      .unwrap_or(false);
    let unstructured_input = properties.is_none()
      && input.get("type").and_then(Value::as_str) == Some("object")
      && input.get("additionalProperties") != Some(&Value::Bool(false));
    let required = input.get("required").and_then(Value::as_array);
    let output_name = format!("{}Result", type_name(&op.name));
    let output_ty = self.output_type(
      &op.output_schema,
      &op.output_schema,
      &output_name,
      &op.name,
      &mut types,
    )?;
    let mut field = async_graphql::dynamic::Field::new(
      &op.name,
      output_ty,
      crate::service::operation_resolver(
        op.name.clone(),
        unstructured_input || oneof_input,
        op.input_schema.clone(),
        op.output_schema.clone(),
      ),
    )
    .description(op.description.clone());
    if oneof_input {
      let ty = self.input_type(
        &op.input_schema,
        &op.input_schema,
        &format!("{}Input", type_name(&op.name)),
        &mut types,
      )?;
      field = field.argument(async_graphql::dynamic::InputValue::new("input", ty));
    } else if let Some(properties) = properties {
      for (name, property) in properties {
        if !valid_name(name) {
          return Err(format!("invalid GraphQL argument {name} on {}", op.name));
        }
        let ty = self.input_type(
          &op.input_schema,
          property,
          &format!("{}{}Input", type_name(&op.name), type_name(name)),
          &mut types,
        )?;
        let is_required = required.is_some_and(|req| req.iter().any(|v| v.as_str() == Some(name)))
          && !crate::refs::accepts_null(&op.input_schema, property)?;
        let ty = if is_required {
          async_graphql::dynamic::TypeRef::NonNull(Box::new(ty))
        } else {
          ty
        };
        field = field.argument(async_graphql::dynamic::InputValue::new(name, ty));
      }
    } else if unstructured_input {
      let ty = self.input_type(
        &op.input_schema,
        &input,
        &format!("{}Input", type_name(&op.name)),
        &mut types,
      )?;
      field = field.argument(async_graphql::dynamic::InputValue::new("input", ty));
    }
    Ok((field, types))
  }
}

pub(crate) fn valid_name(value: &str) -> bool {
  if value.starts_with("__") {
    return false;
  }
  let mut chars = value.chars();
  matches!(chars.next(), Some('_' | 'a'..='z' | 'A'..='Z'))
    && chars.all(|ch| matches!(ch, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
}

pub(crate) fn type_name(value: &str) -> String {
  let mut name = String::new();
  for part in value.split(|ch: char| !ch.is_ascii_alphanumeric()) {
    if part.is_empty() {
      continue;
    }
    let mut chars = part.chars();
    if let Some(first) = chars.next() {
      name.extend(first.to_uppercase());
    }
    name.extend(chars);
  }
  if name.is_empty() {
    "Anonymous".into()
  } else if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
    format!("T{name}")
  } else {
    name
  }
}
