use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::{
  body::{Body, to_bytes},
  http::{Request, StatusCode},
};
use gity_api_contracts::{
  ApiContext, ApiError, ApiResult, ApiTransport, ApplicationApi, ApplicationApiHandle,
  OperationContract, OperationKind,
};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::{GraphqlConfig, GraphqlService, schema::build_schema};

mod schema_tests;

#[derive(Default)]
struct MockApi {
  contexts: Mutex<Vec<(Option<String>, String, String)>>,
  calls: Mutex<Vec<(String, Value)>>,
}

#[async_trait]
impl ApplicationApi for MockApi {
  fn operations(&self) -> Arc<[OperationContract]> {
    contracts().into()
  }

  async fn authenticate(&self, context: &ApiContext) -> ApiResult<()> {
    self.contexts.lock().unwrap().push((
      context.authorization().map(str::to_owned),
      context.language.clone(),
      context.request_id.clone(),
    ));
    if context.authorization().is_none() {
      return Err(ApiError::new(401, "UNAUTHENTICATED", "sign in required"));
    }
    Ok(())
  }

  async fn invoke(
    &self,
    context: &ApiContext,
    operation: &str,
    arguments: Value,
  ) -> ApiResult<Value> {
    if context.authorization().is_none() {
      return Err(ApiError::new(401, "UNAUTHENTICATED", "sign in required"));
    }
    self
      .calls
      .lock()
      .unwrap()
      .push((operation.to_owned(), arguments.clone()));
    if operation == "fail" {
      return Err(ApiError::new(409, "CONFLICT", "operation failed"));
    }
    if operation == "internalFail" {
      return Err(ApiError::new(500, "INTERNAL", "database password leaked"));
    }
    if operation == "project" {
      return Ok(json!({"id": arguments["id"], "name": "sample", "owner": {"name": "Ada"}}));
    }
    Ok(json!({"id": "9007199254740993", "name": arguments["name"]}))
  }
}

fn contracts() -> Vec<OperationContract> {
  vec![
    OperationContract {
      name: "project".into(),
      description: "Get one project".into(),
      group: "projects".into(),
      kind: OperationKind::Query,
      input_schema: json!({"type":"object","properties":{"id":{"type":"string","format":"snowflake-id"}},"required":["id"]}),
      output_schema: json!({"type":"object","properties":{
        "id":{"type":"string","format":"snowflake-id"},"name":{"type":"string"},
        "owner":{"type":"object","properties":{"name":{"type":"string"}}}
      },"required":["id","name"]}),
      destructive: false,
      idempotent: true,
    },
    OperationContract {
      name: "createProject".into(),
      description: "Create a project".into(),
      group: "projects".into(),
      kind: OperationKind::Mutation,
      input_schema: json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}),
      output_schema: json!({"type":"object","properties":{"id":{"type":"string"},"name":{"type":"string"}}}),
      destructive: false,
      idempotent: false,
    },
    OperationContract {
      name: "fail".into(),
      description: "Test failure".into(),
      group: "test".into(),
      kind: OperationKind::Query,
      input_schema: json!({"type":"object","properties":{}}),
      output_schema: json!({"type":"string"}),
      destructive: false,
      idempotent: true,
    },
    OperationContract {
      name: "internalFail".into(),
      description: "Test internal failure".into(),
      group: "test".into(),
      kind: OperationKind::Query,
      input_schema: json!({"type":"object","properties":{}}),
      output_schema: json!({"type":"string"}),
      destructive: false,
      idempotent: true,
    },
  ]
}

fn service(api: Arc<MockApi>) -> GraphqlService {
  let config = GraphqlConfig {
    introspection_enabled: true,
    ..GraphqlConfig::default()
  };
  let schema = build_schema(
    &api.operations(),
    config.max_depth.get(),
    config.max_complexity.get(),
    true,
  )
  .unwrap();
  GraphqlService {
    api: ApplicationApiHandle(api),
    schema,
    max_request_bytes: config.max_request_bytes,
  }
}

fn context(authorization: Option<&str>, language: &str, request_id: &str) -> ApiContext {
  ApiContext::new(
    authorization.map(str::to_owned),
    language.into(),
    request_id.into(),
    ApiTransport::Graphql,
  )
}

#[tokio::test]
async fn generated_schema_exposes_typed_query_mutation_and_string_ids() {
  let api = Arc::new(MockApi::default());
  let service = service(api.clone());
  let schema_response = service.schema.execute("{ __type(name: \"Query\") { fields { name args { name type { kind name ofType { name } } } } } }").await.into_result().unwrap();
  let rendered = schema_response.data.into_json().unwrap();
  assert_eq!(rendered["__type"]["fields"][0]["name"], "project");
  assert_eq!(
    rendered["__type"]["fields"][0]["args"][0]["type"]["ofType"]["name"],
    "ID"
  );
  assert!(
    rendered["__type"]["fields"]
      .as_array()
      .unwrap()
      .iter()
      .all(|field| field["name"] != "execute")
  );

  let response = service
    .execute(
      async_graphql::Request::new(
        "{ project(id: \"9007199254740993\") { id name owner { name } } }",
      ),
      context(Some("Bearer token-a"), "en", "request-a"),
    )
    .await
    .into_result()
    .unwrap();
  assert_eq!(
    response.data.into_json().unwrap(),
    json!({"project":{"id":"9007199254740993","name":"sample","owner":{"name":"Ada"}}})
  );
  {
    let calls = api.calls.lock().unwrap();
    assert_eq!(calls[0].0, "project");
    assert_eq!(calls[0].1, json!({"id":"9007199254740993"}));
  }
  let mutation = service
    .execute(
      async_graphql::Request::new("mutation { createProject(name: \"demo\") { id name } }"),
      context(Some("Bearer token-a"), "en", "request-b"),
    )
    .await
    .into_result()
    .unwrap();
  assert_eq!(
    mutation.data.into_json().unwrap()["createProject"]["name"],
    "demo"
  );
  assert_eq!(api.calls.lock().unwrap()[1].0, "createProject");
}

#[tokio::test]
async fn each_http_request_authenticates_with_its_own_headers_and_language() {
  let api = Arc::new(MockApi::default());
  let app = service(api.clone()).router();
  for (authorization, language) in [("Bearer one", "fr"), ("Bearer two", "zh-CN")] {
    let request = Request::builder()
      .method("POST")
      .uri("/api/graphql")
      .header("content-type", "application/json")
      .header("authorization", authorization)
      .header("accept-language", language)
      .body(Body::from(r#"{"query":"{ project(id: \"9\") { id } }"}"#))
      .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 4096).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["data"]["project"]["id"], "9");
  }
  let contexts = api.contexts.lock().unwrap();
  assert_eq!(contexts.len(), 2);
  assert_eq!(contexts[0].0.as_deref(), Some("Bearer one"));
  assert_eq!(contexts[1].0.as_deref(), Some("Bearer two"));
  assert_eq!(contexts[0].1, "fr");
  assert_eq!(contexts[1].1, "zh-CN");
  assert_ne!(contexts[0].2, contexts[1].2);
}

#[tokio::test]
async fn auth_backend_and_validation_errors_have_stable_extensions() {
  let api = Arc::new(MockApi::default());
  let service = service(api.clone());
  let unauthenticated = service
    .execute(
      async_graphql::Request::new("{ project(id: \"1\") { id } }"),
      context(None, "en", "r1"),
    )
    .await;
  assert_eq!(
    unauthenticated.errors[0]
      .extensions
      .as_ref()
      .unwrap()
      .get("code")
      .unwrap()
      .clone()
      .into_json()
      .unwrap(),
    json!("UNAUTHENTICATED")
  );
  assert_eq!(
    unauthenticated.errors[0]
      .extensions
      .as_ref()
      .unwrap()
      .get("status")
      .unwrap()
      .clone()
      .into_json()
      .unwrap(),
    json!(401)
  );

  let failed = service
    .execute(
      async_graphql::Request::new("{ fail }"),
      context(Some("Bearer token"), "en", "r2"),
    )
    .await;
  assert_eq!(
    failed.errors[0]
      .extensions
      .as_ref()
      .unwrap()
      .get("code")
      .unwrap()
      .clone()
      .into_json()
      .unwrap(),
    json!("CONFLICT")
  );
  assert_eq!(
    failed.errors[0]
      .extensions
      .as_ref()
      .unwrap()
      .get("status")
      .unwrap()
      .clone()
      .into_json()
      .unwrap(),
    json!(409)
  );
  let invalid = service
    .schema
    .execute(
      async_graphql::Request::new("query($id: Int!) { project(id: $id) { id } }")
        .variables(async_graphql::Variables::from_json(json!({"id": 1}))),
    )
    .await;
  assert!(!invalid.errors.is_empty());
}
