use serde_json::json;

#[tokio::test]
#[cfg(not(feature = "api-docs"))]
async fn release_policy_excludes_docs_even_with_spa_fallback_and_preserves_lfs() {
  use axum::{
    body::Body,
    http::{Request, StatusCode},
  };
  use tower::ServiceExt;
  let (directory, mut state) = crate::features::test_support::fixture().await;
  std::sync::Arc::make_mut(&mut state.settings).web_root = directory.path().join("web");
  tokio::fs::create_dir_all(&state.settings.web_root)
    .await
    .unwrap();
  tokio::fs::write(state.settings.web_root.join("index.html"), "<h1>Gity</h1>")
    .await
    .unwrap();
  let router = crate::router(state);
  for path in [
    "/docs",
    "/docs/",
    "/docs/swagger-ui-bundle.js",
    "/openapi.json",
    "/swagger-ui/index.html",
  ] {
    let response = router
      .clone()
      .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
  }
  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/team/p20.git/info/lfs/objects/batch")
        .header("content-type", "application/vnd.git-lfs+json")
        .header("authorization", "Bearer alice")
        .body(Body::from(
          json!({"operation":"download","objects":[]}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_ne!(
    response.status(),
    StatusCode::NOT_FOUND,
    "real LFS endpoint must remain available"
  );
  drop(directory);
}

#[test]
fn typed_arguments_preserve_numeric_branch_names_and_decode_tuple_ids() {
  let args = super::CallArguments {
    path: vec![json!(123), json!("1234")],
    query: json!({"page":1}),
    body: json!({"id":"456"}),
  };
  let decoded: (i64, String) = args.path().unwrap();
  assert_eq!(decoded, (123, "1234".into()));
  let query: std::collections::HashMap<String, String> = args.query().unwrap();
  assert_eq!(query["page"], "1");
  let body: serde_json::Value = args.body().unwrap();
  assert_eq!(body["id"], "456");
}

#[test]
fn rest_context_preserves_explicit_invalid_auth_and_job_credentials() {
  let mut original = axum::http::HeaderMap::new();
  original.insert("authorization", "Invalid credentials".parse().unwrap());
  original.insert("job-token", "runner-test-secret".parse().unwrap());
  let context = super::context(&original, super::ApiTransport::Rest);
  let restored = super::headers(&context);
  assert_eq!(restored["authorization"], original["authorization"]);
  assert_eq!(restored["job-token"], original["job-token"]);
  let debug = format!("{context:?}");
  assert!(!debug.contains("runner-test-secret"));
  assert!(!debug.contains("Invalid credentials"));
}
