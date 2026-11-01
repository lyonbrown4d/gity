use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use gity_core::ObservabilityConfig;
use opentelemetry::{global, propagation::Extractor, trace::TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use tracing::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;
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
struct Headers<'a>(&'a axum::http::HeaderMap);
impl Extractor for Headers<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }
    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|v| v.as_str()).collect()
    }
}
pub async fn request(
    axum::extract::State(state): axum::extract::State<crate::AppState>,
    request: Request,
    next: Next,
) -> Response {
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|v| v.as_str())
        .unwrap_or("unmatched")
        .to_owned();
    let method = match request.method().as_str() {
        "GET" => "GET",
        "POST" => "POST",
        "PUT" => "PUT",
        "PATCH" => "PATCH",
        "DELETE" => "DELETE",
        "HEAD" => "HEAD",
        "OPTIONS" => "OPTIONS",
        _ => "OTHER",
    };
    let id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .filter(|v| {
            v.len() <= 64
                && !v.is_empty()
                && v.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
        .map(str::to_owned)
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
    let context = global::get_text_map_propagator(|p| p.extract(&Headers(request.headers())));
    let span = tracing::info_span!("http.request",request_id=%id,http.route=%route,http.request.method=method,otel.kind="server",http.response.status_code=tracing::field::Empty);
    let _ = span.set_parent(context);
    let started = std::time::Instant::now();
    let actor = crate::auth::principal(&state, request.headers())
        .await
        .ok()
        .map(|p| p.user_id);
    let language = gity_core::locale::negotiate_language(
        request
            .headers()
            .get(axum::http::header::ACCEPT_LANGUAGE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("en"),
    );
    let mut response = gity_core::locale::with_language(
        language,
        gity_core::audit_context(actor, next.run(request).instrument(span.clone())),
    )
    .await;
    response.headers_mut().insert(
        axum::http::header::CONTENT_LANGUAGE,
        axum::http::HeaderValue::from_static(language),
    );
    response.headers_mut().append(
        axum::http::header::VARY,
        axum::http::HeaderValue::from_static("Accept-Language"),
    );
    let status = response.status().as_u16();
    span.record("http.response.status_code", status);
    metrics::counter!("gity_http_requests_total","route"=>route.clone(),"method"=>method,"status"=>status.to_string()).increment(1);
    metrics::histogram!("gity_http_request_duration_seconds","route"=>route,"method"=>method)
        .record(started.elapsed().as_secs_f64());
    if let Ok(id) = id.parse() {
        response.headers_mut().insert("x-request-id", id);
    }
    response
}
