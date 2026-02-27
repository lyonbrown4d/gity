use axum::http::HeaderName;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info_span};
use crate::http::app_state::AppState;

const REQUEST_ID_HEADER: &str = "x-request-id";

// 构造中间件
pub fn apply_request_id_middleware(router: Router<AppState>) -> Router<AppState> {
    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);

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

    router.layer(middleware)
}