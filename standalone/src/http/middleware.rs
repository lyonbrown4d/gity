use crate::http::app_state::AppState;
use axum::Router;
use axum::http::HeaderName;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info_span};

const REQUEST_ID_HEADER: &str = "x-request-id";

// 构造中间件
pub fn apply_request_id_middleware(
  router: Router<AppState>,
  cors_allowed_origins: Option<Vec<String>>,
) -> Router<AppState> {
  let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);
  let cors_layer = build_cors_layer(cors_allowed_origins);

  let middleware = ServiceBuilder::new()
    .layer(SetRequestIdLayer::new(
      x_request_id.clone(),
      MakeRequestUuid,
    ))
    .layer(
      TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
        match request.headers().get(REQUEST_ID_HEADER) {
          Some(request_id) => info_span!("http_request", request_id = ?request_id),
          None => {
            error!("could not extract request_id");
            info_span!("http_request")
          }
        }
      }),
    )
    .layer(PropagateRequestIdLayer::new(x_request_id));

  router.layer(cors_layer).layer(middleware)
}

fn build_cors_layer(cors_allowed_origins: Option<Vec<String>>) -> CorsLayer {
  let base = CorsLayer::new()
    .allow_methods([
      Method::GET,
      Method::POST,
      Method::PUT,
      Method::PATCH,
      Method::DELETE,
      Method::OPTIONS,
    ])
    .allow_headers([
      AUTHORIZATION,
      CONTENT_TYPE,
      HeaderName::from_static(REQUEST_ID_HEADER),
    ])
    .expose_headers([HeaderName::from_static(REQUEST_ID_HEADER)]);

  let origins = cors_allowed_origins
    .unwrap_or_default()
    .into_iter()
    .filter_map(|origin| HeaderValue::from_str(origin.trim()).ok())
    .collect::<Vec<_>>();

  if origins.is_empty() {
    base.allow_origin(Any)
  } else {
    base.allow_origin(origins)
  }
}
