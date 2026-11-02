use std::sync::Arc;

use async_graphql::{
  Error as GqlError, ErrorExtensions, Pos, Request as GqlRequest, Response as GqlResponse,
  dynamic::{FieldFuture, Schema},
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{Router, extract::State, http::HeaderMap, response::IntoResponse, routing::post};
use fluxdi::{Injector, Provider, Shared, module::Module};
use gity_api_contracts::{ApiContext, ApiError, ApiTransport, ApplicationApiHandle};
use serde_json::{Map, Value};
use tower_http::limit::RequestBodyLimitLayer;
use uuid::Uuid;

use crate::{GraphqlConfig, schema::build_schema};

struct RequestContext(ApiContext);

/// GraphQL adapter provider. The parent application composes it into its injector.
pub struct GraphqlModule {
  api: ApplicationApiHandle,
  schema: Schema,
  max_request_bytes: usize,
}

impl GraphqlModule {
  pub fn new(api: ApplicationApiHandle, config: GraphqlConfig) -> Result<Self, String> {
    config.validate()?;
    // Build once at configuration time to reject invalid contracts before serving.
    let schema = build_schema(
      &api.0.operations(),
      config.max_depth.get(),
      config.max_complexity.get(),
      config.introspection_enabled,
    )?;
    Ok(Self {
      api,
      schema,
      max_request_bytes: config.max_request_bytes,
    })
  }
}

impl Module for GraphqlModule {
  fn configure(&self, injector: &Injector) -> Result<(), fluxdi::Error> {
    let service = GraphqlService {
      api: self.api.clone(),
      schema: self.schema.clone(),
      max_request_bytes: self.max_request_bytes,
    };
    injector.try_provide(Provider::<GraphqlService>::root(move |_| {
      Shared::new(service.clone())
    }))
  }
}

/// Stateless GraphQL service that exposes a router for an enclosing server.
#[derive(Clone)]
pub struct GraphqlService {
  pub(super) api: ApplicationApiHandle,
  pub(super) schema: Schema,
  pub(super) max_request_bytes: usize,
}

impl GraphqlService {
  /// Mounts only `/api/graphql`; the service does not start its own listener.
  pub fn router(&self) -> Router {
    Router::new()
      .route("/api/graphql", post(graphql_handler))
      .layer(RequestBodyLimitLayer::new(self.max_request_bytes))
      .with_state(Arc::new(self.clone()))
  }

  pub(super) async fn execute(&self, request: GqlRequest, context: ApiContext) -> GqlResponse {
    match self.api.0.authenticate(&context).await {
      Ok(()) => {
        self
          .schema
          .execute(request.data(RequestContext(context)).data(self.api.clone()))
          .await
      }
      Err(error) => api_error_response(error),
    }
  }
}

async fn graphql_handler(
  State(service): State<Arc<GraphqlService>>,
  headers: HeaderMap,
  request: GraphQLRequest,
) -> impl IntoResponse {
  let authorization = headers
    .get(axum::http::header::AUTHORIZATION)
    .and_then(|value| value.to_str().ok())
    .map(str::to_owned);
  let language = headers
    .get(axum::http::header::ACCEPT_LANGUAGE)
    .and_then(|value| value.to_str().ok())
    .unwrap_or("en")
    .to_owned();
  let context = ApiContext::new(
    authorization,
    language,
    Uuid::now_v7().to_string(),
    ApiTransport::Graphql,
  );
  GraphQLResponse::from(service.execute(request.into_inner(), context).await)
}

pub(crate) fn operation_resolver(
  operation: String,
  flatten_input: bool,
  input_schema: Value,
  output_schema: Value,
) -> impl for<'a> Fn(async_graphql::dynamic::ResolverContext<'a>) -> FieldFuture<'a>
+ Send
+ Sync
+ 'static {
  move |ctx| {
    let operation = operation.clone();
    let input_schema = input_schema.clone();
    let output_schema = output_schema.clone();
    FieldFuture::new(async move {
      let request = ctx.data::<RequestContext>()?;
      let mut arguments = Map::new();
      for (name, value) in ctx.args.iter() {
        let json = value
          .as_value()
          .clone()
          .into_json()
          .map_err(|error| GqlError::new(error.to_string()))?;
        arguments.insert(name.to_string(), json);
      }
      let arguments = if flatten_input && arguments.len() == 1 {
        arguments.remove("input").unwrap_or(Value::Null)
      } else {
        Value::Object(arguments)
      };
      let arguments = crate::input::normalize_input(&input_schema, &input_schema, arguments)
        .map_err(GqlError::new)?;
      let backend = ctx.data::<ApplicationApiHandle>()?;
      let result = backend
        .0
        .invoke(&request.0, &operation, arguments)
        .await
        .map_err(api_error)?;
      crate::output::json_field_value(
        result,
        &output_schema,
        &output_schema,
        &async_graphql::dynamic::TypeRef::named(format!(
          "{}Result",
          crate::schema::type_name(&operation)
        )),
      )
    })
  }
}

fn api_error(error: ApiError) -> GqlError {
  let message = if error.status >= 500 {
    "Operation failed"
  } else {
    error.message.as_str()
  };
  GqlError::new(message).extend_with(|_, extensions| {
    extensions.set("code", error.code);
    extensions.set("status", i64::from(error.status));
  })
}

fn api_error_response(error: ApiError) -> GqlResponse {
  let error = api_error(error).into_server_error(Pos::default());
  GqlResponse::from_errors(vec![error])
}
