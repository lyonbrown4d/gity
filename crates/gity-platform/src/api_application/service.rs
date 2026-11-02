use super::{AppState, CallArguments, catalog, headers, output};
use async_trait::async_trait;
use gity_api_contracts::{
  ApiContext, ApiResult, ApiTransport, ApplicationApi, OperationContract, OperationKind,
};
use serde_json::{Value, json};
use std::sync::Arc;

pub(crate) struct PlatformApi {
  state: AppState,
  operations: Vec<catalog::CatalogOperation>,
  contracts: Arc<[OperationContract]>,
  validators: Vec<jsonschema::Validator>,
}
impl PlatformApi {
  pub fn new(state: AppState) -> Result<Self, String> {
    let operations = catalog::build()?;
    let contracts = operations.iter().map(|op| op.contract.clone()).collect();
    let validators = operations
      .iter()
      .map(|op| {
        jsonschema::validator_for(&op.contract.input_schema)
          .map_err(|_| format!("invalid application schema: {}", op.contract.name))
      })
      .collect::<Result<Vec<_>, _>>()?;
    Ok(Self {
      state,
      operations,
      contracts,
      validators,
    })
  }
  async fn oauth_principal(
    &self,
    context: &ApiContext,
  ) -> ApiResult<gity_identity::OAuthPrincipal> {
    let token = context
      .authorization()
      .and_then(|v| v.strip_prefix("Bearer "))
      .ok_or_else(|| super::output::error(401, "UNAUTHENTICATED", "OAuth bearer token required"))?;
    let audience = format!("{}/mcp", self.state.settings.base_url.trim_end_matches('/'));
    let principal = self
      .state
      .oauth
      .validate_bearer(token, &audience)
      .await
      .map_err(|_| super::output::error(401, "UNAUTHENTICATED", "invalid or expired credentials"))?
      .ok_or_else(|| {
        super::output::error(401, "UNAUTHENTICATED", "invalid or expired credentials")
      })?;
    if !principal.scopes.iter().any(|scope| scope == "mcp") {
      return Err(super::output::error(
        403,
        "INSUFFICIENT_SCOPE",
        "MCP scope required",
      ));
    }
    Ok(principal)
  }
}
#[async_trait]
impl ApplicationApi for PlatformApi {
  fn operations(&self) -> Arc<[OperationContract]> {
    self.contracts.clone()
  }
  async fn authenticate(&self, context: &ApiContext) -> ApiResult<()> {
    if context.transport == ApiTransport::Mcp {
      self.oauth_principal(context).await?;
    } else if context
      .authorization()
      .is_some_and(|v| v.starts_with("Bearer gity_oauth_"))
    {
      return Err(super::output::error(
        401,
        "UNAUTHENTICATED",
        "credential audience does not match this API",
      ));
    }
    crate::auth::principal(&self.state, &headers(context))
      .await
      .map_err(output::convert)?;
    Ok(())
  }
  async fn invoke(
    &self,
    context: &ApiContext,
    name: &str,
    mut arguments: Value,
  ) -> ApiResult<Value> {
    self.authenticate(context).await?;
    let (index, operation) = self
      .operations
      .iter()
      .enumerate()
      .find(|(_, op)| op.contract.name == name)
      .ok_or_else(|| super::output::error(404, "NOT_FOUND", "unknown operation"))?;
    if context.transport == ApiTransport::Mcp {
      let principal = self.oauth_principal(context).await?;
      let required = if operation.contract.kind == OperationKind::Query {
        "read_api"
      } else {
        "write_api"
      };
      if !principal.scopes.iter().any(|scope| scope == required) {
        return Err(super::output::error(
          403,
          "INSUFFICIENT_SCOPE",
          "credential scope does not permit this operation",
        ));
      }
    }
    if !self.validators[index].is_valid(&arguments) {
      return Err(super::output::error(
        400,
        "INVALID_ARGUMENT",
        "arguments do not match operation schema",
      ));
    }
    super::ids::native_arguments(&operation.contract.input_schema, &mut arguments)?;
    let path = operation
      .path_parameters
      .iter()
      .map(|(name, integer)| {
        let value = arguments["path"][name].clone();
        if *integer {
          value
            .as_str()
            .and_then(|v| v.parse::<i64>().ok())
            .map_or(value, |v| json!(v))
        } else {
          value
        }
      })
      .collect();
    let args = CallArguments {
      path,
      query: arguments.get("query").cloned().unwrap_or_else(|| json!({})),
      body: arguments.get("body").cloned().unwrap_or(Value::Null),
    };
    let language = gity_core::locale::negotiate_language(&context.language);
    let span = tracing::info_span!("application.operation", operation = %name, transport = ?context.transport, request_id = %context.request_id);
    use tracing::Instrument;
    let actor = crate::auth::principal(&self.state, &headers(context))
      .await
      .map_err(output::convert)?
      .user_id;
    let result = gity_core::audit_context(
      Some(actor),
      gity_core::locale::with_language(
        language,
        (operation.registration.invoke)(self.state.clone(), context.clone(), args).instrument(span),
      ),
    )
    .await;
    metrics::counter!("gity_api_operations_total", "operation" => name.to_owned(), "result" => if result.is_ok() {"success"} else {"failure"}).increment(1);
    result.map(|value| {
      if let Value::Array(items) = value {
        let page = arguments["pagination"]["page"].as_u64().unwrap_or(1);
        let size = arguments["pagination"]["page_size"]
          .as_u64()
          .unwrap_or(50)
          .min(100);
        Value::Array(
          items
            .into_iter()
            .skip(((page - 1) * size) as usize)
            .take(size as usize)
            .collect(),
        )
      } else {
        value
      }
    })
  }
}
