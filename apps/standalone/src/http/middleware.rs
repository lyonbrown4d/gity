use crate::http::app_state::AppState;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::HeaderName;
use axum::http::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{HeaderValue, Method, Request, StatusCode, response::Parts};
use axum::middleware::{Next, from_fn};
use axum::response::Response;
use domain::api::response::ApiResponse;
use serde_json::Value;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info_span};

const REQUEST_ID_HEADER: &str = "x-request-id";
const MAX_WRAPPED_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

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

  router
    .layer(cors_layer)
    .layer(from_fn(normalize_api_response))
    .layer(middleware)
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

async fn normalize_api_response(request: Request<Body>, next: Next) -> Response {
  let path = request.uri().path().to_string();
  let response = next.run(request).await;
  if !is_api_path(path.as_str()) {
    return response;
  }

  wrap_api_response(response).await
}

fn is_api_path(path: &str) -> bool {
  path == "/api/v1" || path.starts_with("/api/v1/")
}

async fn wrap_api_response(response: Response) -> Response {
  let original_status = response.status();

  if original_status == StatusCode::NO_CONTENT {
    return build_wrapped_response(
      response.into_parts().0,
      StatusCode::OK,
      ApiResponse::success(Value::Null),
    );
  }

  let is_json = response
    .headers()
    .get(CONTENT_TYPE)
    .and_then(|value| value.to_str().ok())
    .map(|value| value.to_ascii_lowercase().starts_with("application/json"))
    .unwrap_or(false);
  if !is_json {
    return response;
  }

  let (parts, body) = response.into_parts();
  let bytes = match to_bytes(body, MAX_WRAPPED_RESPONSE_BYTES).await {
    Ok(value) => value,
    Err(_) => {
      return build_wrapped_response(
        parts,
        StatusCode::INTERNAL_SERVER_ERROR,
        ApiResponse::with_message(
          StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
          "failed to read response body",
          Value::Null,
        ),
      );
    }
  };

  if bytes.is_empty() {
    let wrapped = if original_status.is_success() {
      ApiResponse::success(Value::Null)
    } else {
      ApiResponse::with_message(
        original_status.as_u16() as i32,
        default_error_message(original_status),
        Value::Null,
      )
    };
    return build_wrapped_response(parts, original_status, wrapped);
  }

  let parsed = match serde_json::from_slice::<Value>(&bytes) {
    Ok(value) => value,
    Err(_) => return Response::from_parts(parts, Body::from(bytes)),
  };

  if is_already_wrapped(parsed.as_object()) {
    return Response::from_parts(parts, Body::from(bytes));
  }

  let wrapped = if original_status.is_success() {
    ApiResponse::success(parsed)
  } else {
    let message = parsed
      .as_object()
      .and_then(|object| object.get("message"))
      .and_then(Value::as_str)
      .map(ToString::to_string)
      .unwrap_or_else(|| default_error_message(original_status));
    ApiResponse::with_message(original_status.as_u16() as i32, message, parsed)
  };

  build_wrapped_response(parts, original_status, wrapped)
}

fn is_already_wrapped(object: Option<&serde_json::Map<String, Value>>) -> bool {
  let Some(object) = object else {
    return false;
  };

  object.contains_key("code") && object.contains_key("message") && object.contains_key("data")
}

fn build_wrapped_response(parts: Parts, status: StatusCode, body: ApiResponse<Value>) -> Response {
  let payload = match serde_json::to_vec(&body) {
    Ok(value) => value,
    Err(_) => {
      let fallback = br#"{"code":500,"message":"failed to serialize api response","data":null}"#;
      let mut response = Response::new(Body::from(fallback.as_slice().to_vec()));
      *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
      response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
      );
      response.headers_mut().remove(CONTENT_LENGTH);
      return response;
    }
  };

  let mut response = Response::new(Body::from(payload));
  *response.status_mut() = status;
  *response.headers_mut() = parts.headers;
  response.headers_mut().insert(
    CONTENT_TYPE,
    HeaderValue::from_static("application/json; charset=utf-8"),
  );
  response.headers_mut().remove(CONTENT_LENGTH);
  response
}

fn default_error_message(status: StatusCode) -> String {
  status
    .canonical_reason()
    .map(ToString::to_string)
    .unwrap_or_else(|| "request failed".to_string())
}
