use super::*;

#[test]
fn invalid_operation_names_and_unbounded_limits_are_rejected() {
  let mut invalid = contracts();
  invalid[0].name = "not-valid".into();
  assert!(build_schema(&invalid, 12, 200, false).is_err());
  let config = GraphqlConfig {
    max_depth: std::num::NonZeroUsize::new(100).unwrap(),
    ..GraphqlConfig::default()
  };
  assert!(config.validate().is_err());
}

#[tokio::test]
async fn local_refs_nullable_types_and_oneof_inputs_build_real_graphql_types() {
  let operation = OperationContract {
    name: "lookupThing".into(),
    description: "Typed referenced schemas".into(),
    group: "things".into(),
    kind: OperationKind::Query,
    input_schema: json!({
      "$defs": {
        "Filter": {"type":"object","properties":{"state":{"type":["string","null"],"enum":["OPEN","CLOSED"]}},"required":["state"]},
        "ByName": {"type":"object","properties":{"byName":{"$ref":"#/$defs/Filter"}}},
        "ById": {"type":"object","properties":{"byId":{"type":"string","format":"snowflake-id"}}}
      },
      "type":"object",
      "properties": {
        "filter":{"$ref":"#/$defs/Filter"},
        "access":{"oneOf":[{"$ref":"#/$defs/ByName"},{"$ref":"#/$defs/ById"}]}
      }
    }),
    output_schema: json!({
      "$defs":{"Thing":{"type":"object","properties":{"id":{"type":"string","format":"snowflake-id"}}}},
      "$ref":"#/$defs/Thing"
    }),
    destructive: false,
    idempotent: true,
  };
  let schema = build_schema(std::slice::from_ref(&operation), 12, 200, true).unwrap();
  let response = schema.execute("{ __type(name: \"LookupThingAccessInput\") { kind isOneOf inputFields { name type { kind name ofType { name } } } } }").await.into_result().unwrap();
  let response = response.data.into_json().unwrap();
  assert_eq!(response["__type"]["isOneOf"], true);
  assert_eq!(response["__type"]["inputFields"][0]["name"], "byId");
  assert_eq!(response["__type"]["inputFields"][1]["name"], "byName");
  let filter = schema
    .execute(
      "{ __type(name: \"LookupThingFilterInput\") { inputFields { name type { kind name } } } }",
    )
    .await
    .into_result()
    .unwrap();
  let filter = filter.data.into_json().unwrap();
  assert_eq!(
    filter["__type"]["inputFields"][0]["type"]["name"],
    "LookupThingFilterInputState"
  );
  assert_eq!(filter["__type"]["inputFields"][0]["type"]["kind"], "ENUM");
  let normalized = crate::input::normalize_input(
    &operation.input_schema,
    &operation.input_schema,
    json!({"access":{"byName":{"byName":{"state":"OPEN"}}}}),
  )
  .unwrap();
  assert_eq!(normalized, json!({"access":{"byName":{"state":"OPEN"}}}));
}

#[tokio::test]
async fn default_schema_disables_introspection_and_http_body_size_is_bounded() {
  let api = Arc::new(MockApi::default());
  let config = GraphqlConfig::default();
  let schema = build_schema(
    &api.operations(),
    config.max_depth.get(),
    config.max_complexity.get(),
    config.introspection_enabled,
  )
  .unwrap();
  let introspection = schema.execute("{ __schema { queryType { name } } }").await;
  assert!(!introspection.errors.is_empty());

  let mut graphql = service(api);
  graphql.max_request_bytes = 8;
  let response = graphql
    .router()
    .oneshot(
      axum::http::Request::builder()
        .method("POST")
        .uri("/api/graphql")
        .header("content-type", "application/json")
        .header("content-length", "9")
        .body(axum::body::Body::from("123456789"))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), axum::http::StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn all_of_and_recursive_local_refs_generate_finite_typed_schema() {
  let operation = OperationContract {
    name: "inspectNode".into(),
    description: "Composed input and recursive output".into(),
    group: "nodes".into(),
    kind: OperationKind::Query,
    input_schema: json!({
      "$defs": {
        "Left": {"type":"object","properties":{"left":{"type":"string"}}},
        "Right": {"type":"object","properties":{"right":{"type":"integer"}},"required":["right"]},
        "NodeFilter": {"allOf":[{"$ref":"#/$defs/Left"},{"$ref":"#/$defs/Right"}]}
      },
      "type":"object",
      "properties":{"filter":{"$ref":"#/$defs/NodeFilter"}}
    }),
    output_schema: json!({
      "$defs":{"Node":{"type":"object","properties":{"id":{"type":"string"},"next":{"$ref":"#/$defs/Node"}}}},
      "$ref":"#/$defs/Node"
    }),
    destructive: false,
    idempotent: true,
  };
  let schema = build_schema(&[operation], 12, 200, true).unwrap();
  let introspection = schema
    .execute("{ __type(name: \"InspectNodeFilterInput\") { inputFields { name type { kind name } } } output: __type(name: \"InspectNodeResult\") { fields { name type { kind name ofType { name } } } } }")
    .await
    .into_result()
    .unwrap()
    .data
    .into_json()
    .unwrap();
  let input_fields = introspection["__type"]["inputFields"].as_array().unwrap();
  assert_eq!(input_fields.len(), 2);
  assert!(input_fields.iter().any(|field| field["name"] == "left"));
  assert!(input_fields.iter().any(|field| field["name"] == "right"));
  let output_fields = introspection["output"]["fields"].as_array().unwrap();
  let next = output_fields
    .iter()
    .find(|field| field["name"] == "next")
    .unwrap();
  assert_eq!(next["type"]["name"], "InspectNodeResult");
}

#[tokio::test]
async fn internal_resolver_errors_are_redacted() {
  let api = Arc::new(MockApi::default());
  let response = service(api)
    .execute(
      async_graphql::Request::new("{ internalFail }"),
      context(Some("Bearer token"), "en", "internal-error"),
    )
    .await;
  assert_eq!(response.errors[0].message, "Operation failed");
  assert!(!response.errors[0].message.contains("database password"));
}

#[tokio::test]
async fn query_depth_limit_rejects_deep_selections() {
  let api = Arc::new(MockApi::default());
  let limited = build_schema(&api.operations(), 1, 200, true)
    .unwrap()
    .execute("{ project(id: \"1\") { owner { name } } }")
    .await;
  assert!(!limited.errors.is_empty());
  let api = Arc::new(MockApi::default());
  let limited = build_schema(&api.operations(), 12, 1, true)
    .unwrap()
    .execute("{ first: fail second: fail }")
    .await;
  assert!(!limited.errors.is_empty());
}

#[tokio::test]
async fn graphql_batch_requests_are_rejected() {
  let api = Arc::new(MockApi::default());
  let app = service(api.clone()).router();
  let body = r#"[{"query":"{ fail }"},{"query":"{ fail }"}]"#;
  let response = app
    .oneshot(
      axum::http::Request::builder()
        .method("POST")
        .uri("/api/graphql")
        .header("content-type", "application/json")
        .header("authorization", "Bearer token")
        .body(axum::body::Body::from(body))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
  assert!(api.calls.lock().unwrap().is_empty());
}
