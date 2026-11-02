use crate::McpConfig;
use gity_api_contracts::{ApiContext, ApplicationApi};
use rmcp::model::{
  CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ErrorCode, ErrorData,
  Implementation, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
  ToolAnnotations,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler};
use serde_json::{Map, Value, json};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct McpHandler {
  backend: Arc<dyn ApplicationApi>,
  config: McpConfig,
}

impl McpHandler {
  pub(crate) fn new(backend: Arc<dyn ApplicationApi>, config: McpConfig) -> Self {
    Self { backend, config }
  }

  fn selected_operations(&self) -> Vec<gity_api_contracts::OperationContract> {
    self
      .backend
      .operations()
      .iter()
      .filter(|operation| {
        self
          .config
          .tool_groups
          .as_ref()
          .is_none_or(|groups| groups.contains(&operation.group))
      })
      .cloned()
      .collect()
  }
}

impl ServerHandler for McpHandler {
  fn get_info(&self) -> ServerConfig {
    ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
      .with_server_info(Implementation::new("gity", env!("CARGO_PKG_VERSION")))
  }

  fn list_tools(
    &self,
    _request: Option<PaginatedRequestParams>,
    _context: RequestContext<RoleServer>,
  ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
    let tools = self
      .selected_operations()
      .into_iter()
      .map(tool_from_contract)
      .collect();
    std::future::ready(Ok(ListToolsResult::with_all_items(tools)))
  }

  async fn call_tool(
    &self,
    request: CallToolRequestParams,
    context: RequestContext<RoleServer>,
  ) -> Result<CallToolResponse, ErrorData> {
    let Some(operation) = self
      .selected_operations()
      .into_iter()
      .find(|operation| operation.name == request.name.as_ref())
    else {
      return Err(ErrorData::new(
        ErrorCode::METHOD_NOT_FOUND,
        "unknown MCP tool",
        Some(json!({"code": "unknown_operation"})),
      ));
    };
    let Some(parts) = context.extensions.get::<http::request::Parts>() else {
      return Err(ErrorData::internal_error(
        "request context is unavailable",
        None,
      ));
    };
    let Some(api_context) = parts.extensions.get::<ApiContext>() else {
      return Err(ErrorData::internal_error(
        "request context is unavailable",
        None,
      ));
    };
    let arguments = request
      .arguments
      .map(Value::Object)
      .unwrap_or_else(|| json!({}));
    match self
      .backend
      .invoke(api_context, &operation.name, arguments)
      .await
    {
      Ok(value) => Ok(CallToolResult::structured(json!({"result": value})).into()),
      Err(error) => {
        let message = if error.status >= 500 {
          "Operation failed".to_owned()
        } else {
          concise_message(&error.message)
        };
        let result = CallToolResult::error(vec![ContentBlock::text(message.clone())]);
        let mut result = result;
        result.structured_content = Some(json!({
          "error": { "code": error.code, "message": message }
        }));
        Ok(result.into())
      }
    }
  }
}

fn concise_message(message: &str) -> String {
  let cleaned = message
    .chars()
    .filter(|character| !character.is_control())
    .take(256)
    .collect::<String>();
  if cleaned.is_empty() {
    "Operation failed".into()
  } else {
    cleaned
  }
}

fn tool_from_contract(contract: gity_api_contracts::OperationContract) -> Tool {
  let mut tool = Tool::default();
  tool.name = contract.name.clone().into();
  tool.description = Some(contract.description.into());
  tool.input_schema = Arc::new(input_schema(contract.input_schema));
  tool.output_schema = Some(Arc::new(wrapped_output_schema(contract.output_schema)));
  let mut annotations = ToolAnnotations::new();
  annotations.title = Some(contract.name);
  annotations.read_only_hint = Some(matches!(
    contract.kind,
    gity_api_contracts::OperationKind::Query
  ));
  annotations.destructive_hint = Some(contract.destructive);
  annotations.idempotent_hint = Some(contract.idempotent);
  tool.annotations = Some(annotations);
  tool
}

fn wrapped_output_schema(mut value: Value) -> Map<String, Value> {
  let definitions = value
    .as_object_mut()
    .and_then(|object| object.remove("$defs"));
  let mut properties = Map::new();
  properties.insert("result".into(), value);
  let mut schema = Map::new();
  schema.insert("type".into(), Value::String("object".into()));
  schema.insert("properties".into(), Value::Object(properties));
  schema.insert("required".into(), serde_json::json!(["result"]));
  if let Some(definitions) = definitions {
    schema.insert("$defs".into(), definitions);
  }
  schema
}

fn input_schema(value: Value) -> Map<String, Value> {
  let mut schema = value.as_object().cloned().unwrap_or_default();
  if schema.get("type").is_none() && (schema.contains_key("oneOf") || schema.contains_key("anyOf"))
  {
    schema.insert("type".into(), Value::String("object".into()));
  }
  schema
}

#[cfg(test)]
mod tests;
