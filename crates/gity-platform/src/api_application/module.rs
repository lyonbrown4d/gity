use super::{AppState, PlatformApi};
use fluxdi::{Injector, Provider, Shared, module::Module};
use gity_api_contracts::ApplicationApiHandle;
use gity_graphql::{GraphqlConfig, GraphqlModule, GraphqlService};
use gity_mcp::{McpConfig, McpModule, McpService};
use std::{sync::Arc, time::Duration};

#[derive(Clone)]
pub(crate) struct ApiRoutes(pub axum::Router);

/// Compose both adapters in the platform's existing FluxDI container.
pub(crate) fn configure(state: &AppState, injector: &Injector) -> Result<(), fluxdi::Error> {
  let make_error = |message: String| fluxdi::Error::factory_failed("ApplicationApi", &message);
  let backend = ApplicationApiHandle(Arc::new(
    PlatformApi::new(state.clone()).map_err(make_error)?,
  ));
  let handle = Shared::new(backend.clone());
  injector.try_provide(Provider::<ApplicationApiHandle>::root(move |_| {
    handle.clone()
  }))?;
  let config = &state.settings.api;
  let mut router = axum::Router::new();
  if config.graphql_enabled {
    let module = GraphqlModule::new(
      backend.clone(),
      GraphqlConfig {
        max_depth: config
          .graphql_max_depth
          .try_into()
          .map_err(|_| make_error("invalid GraphQL depth".into()))?,
        max_complexity: config
          .graphql_max_complexity
          .try_into()
          .map_err(|_| make_error("invalid GraphQL complexity".into()))?,
        max_request_bytes: config.max_request_bytes,
        introspection_enabled: config.graphql_introspection,
      },
    )
    .map_err(make_error)?;
    module.configure(injector)?;
    router = router.merge(injector.try_resolve::<GraphqlService>()?.router());
  }
  if config.mcp_enabled {
    let base = state.settings.base_url.trim_end_matches('/');
    let origin =
      reqwest::Url::parse(base).map_err(|_| make_error("invalid HTTP base URL".into()))?;
    let host = origin
      .host_str()
      .ok_or_else(|| make_error("invalid HTTP base host".into()))?;
    let mut mcp = McpConfig {
      oauth_resource_metadata_url: format!("{base}/.well-known/oauth-protected-resource/mcp"),
      allowed_origins: config.mcp_allowed_origins.iter().cloned().collect(),
      allowed_hosts: config.mcp_allowed_hosts.clone(),
      max_request_body_bytes: config.max_request_bytes,
      max_concurrent_requests: config.mcp_max_concurrent_requests,
      request_timeout: Duration::from_secs(config.mcp_request_timeout_seconds),
      ..McpConfig::default()
    };
    if mcp.allowed_origins.is_empty() {
      mcp
        .allowed_origins
        .insert(origin.origin().ascii_serialization());
    }
    if mcp.allowed_hosts.is_empty() {
      mcp.allowed_hosts.push(host.to_owned());
    }
    let module = McpModule::new(Shared::new(backend), Shared::new(mcp));
    module.configure(injector)?;
    router = router.merge(injector.try_resolve::<McpService>()?.router());
  }
  let routes = Shared::new(ApiRoutes(router));
  injector.try_provide(Provider::<ApiRoutes>::root(move |_| routes.clone()))
}
