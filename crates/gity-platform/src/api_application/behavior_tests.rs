use super::*;
use gity_api_contracts::{ApiContext, ApplicationApi};
use serde_json::json;

fn context(user: &str) -> ApiContext {
  ApiContext::new(
    Some(format!("Bearer {user}")),
    "en".into(),
    uuid::Uuid::now_v7().to_string(),
    ApiTransport::Graphql,
  )
}
#[tokio::test]
async fn adapters_compose_real_catalog_in_the_platform_container() {
  let (_directory, state) = crate::features::test_support::fixture().await;
  let injector = fluxdi::Injector::root();
  module::configure(&state, &injector).unwrap();
  assert!(
    injector
      .try_resolve::<gity_graphql::GraphqlService>()
      .is_ok()
  );
  assert!(injector.try_resolve::<gity_mcp::McpService>().is_ok());
}

#[tokio::test]
async fn shared_operations_enforce_existing_identity_and_project_permissions() {
  let (_directory, state) = crate::features::test_support::fixture().await;
  let catalog = catalog::build().unwrap();
  let operation = catalog
    .iter()
    .find(|entry| entry.registration.handler.ends_with("::issues::list"))
    .unwrap();
  let name = operation.contract.name.clone();
  let api = PlatformApi::new(state.clone()).unwrap();
  let arguments = json!({"path":{"id":"20"}});
  let owner = api
    .invoke(&context("alice"), &name, arguments.clone())
    .await
    .unwrap();
  assert!(owner.is_array());
  let denied = api
    .invoke(&context("outsider"), &name, arguments.clone())
    .await
    .unwrap_err();
  assert!([403, 404].contains(&denied.status));
  let invalid = api
    .invoke(&context("alice"), &name, json!({"path":{"id":"not-an-id"}}))
    .await
    .unwrap_err();
  assert_eq!(invalid.status, 400);
  let anonymous = ApiContext::new(None, "en".into(), "request".into(), ApiTransport::Graphql);
  assert_eq!(
    api
      .invoke(&anonymous, &name, arguments)
      .await
      .unwrap_err()
      .status,
    401
  );
  assert_eq!(
    api
      .authenticate(&ApiContext::new(
        Some("Bearer alice".into()),
        "en".into(),
        "request".into(),
        ApiTransport::Mcp
      ))
      .await
      .unwrap_err()
      .status,
    401
  );
}
