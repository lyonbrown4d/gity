//! Shared tracing, Prometheus and OTLP setup for independently deployed components.
mod config;
mod request;
pub use config::ObservabilityConfig;
use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
pub use request::instrument;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};
pub struct TelemetryGuard {
  provider: Option<SdkTracerProvider>,
}
impl TelemetryGuard {
  pub fn shutdown(self) {
    if let Some(provider) = self.provider
      && let Err(error) = provider.shutdown()
    {
      tracing::error!(error=%error,"trace exporter shutdown failed");
    }
  }
}

pub fn init(config: &ObservabilityConfig) -> anyhow::Result<TelemetryGuard> {
  config.validate()?;
  let provider = if config.otlp_endpoint.is_empty() {
    None
  } else {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
      .with_http()
      .with_endpoint(&config.otlp_endpoint)
      .build()?;
    Some(
      SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
          Resource::builder()
            .with_service_name(config.service_name.clone())
            .build(),
        )
        .build(),
    )
  };
  let otel = provider
    .as_ref()
    .map(|p| tracing_opentelemetry::layer().with_tracer(p.tracer("gity")));
  let formatting = if config.log_format == "json" {
    tracing_subscriber::fmt::layer().json().boxed()
  } else {
    tracing_subscriber::fmt::layer().boxed()
  };
  let filter = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or(tracing_subscriber::EnvFilter::try_new(&config.log_level)?);
  tracing_subscriber::registry()
    .with(filter)
    .with(formatting)
    .with(otel)
    .try_init()?;
  global::set_text_map_propagator(opentelemetry_sdk::propagation::TraceContextPropagator::new());
  if config.metrics_enabled {
    let address = config.metrics_address.parse::<std::net::SocketAddr>()?;
    metrics_exporter_prometheus::PrometheusBuilder::new()
      .with_http_listener(address)
      .install()?;
    metrics::describe_counter!(
      "gity_http_requests_total",
      "HTTP requests by route, method and status"
    );
    metrics::describe_histogram!(
      "gity_http_request_duration_seconds",
      "Time until response headers in seconds"
    );
    metrics::describe_counter!(
      "gity_scheduler_ticks_total",
      "Pipeline scheduler tick outcomes"
    );
  }
  Ok(TelemetryGuard { provider })
}
