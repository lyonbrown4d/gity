use crate::app_state::init;
use crate::cli::Args;
use crate::rest::auth::__path_login;
use crate::rest::auth::login;
use crate::rest::openapi::{openapi, ApiDoc};
use crate::rest::user::__path_create_user;
use crate::rest::user::__path_root;
use crate::rest::user::{create_user, root};
use axum::http::{HeaderName, Request};
use axum::{
    routing::{get, post},
    Json,
};
use clap::Parser;
use std::net::Ipv4Addr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info, info_span};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

mod app_state;
mod cli;
mod configuration;
mod rest;
const REQUEST_ID_HEADER: &str = "x-request-id";

#[tokio::main]
async fn main() {
  let app_state = init().await;
  info!("{:?}", app_state);
  let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);

  let middleware = ServiceBuilder::new()
    .layer(SetRequestIdLayer::new(
      x_request_id.clone(),
      MakeRequestUuid,
    ))
    .layer(
      TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
        // Log the request id as generated.
        let request_id = request.headers().get(REQUEST_ID_HEADER);

        match request_id {
          Some(request_id) => info_span!(
              "http_request",
              request_id = ?request_id,
          ),
          None => {
            error!("could not extract request_id");
            info_span!("http_request")
          }
        }
      }),
    )
    .layer(PropagateRequestIdLayer::new(x_request_id));
  let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
    .routes(routes![root, create_user])
    .routes(routes![login])
    .route("/api-docs/openapi.json", get(openapi))
    .with_state(app_state)
    .split_for_parts();

  let router = router
      .layer(middleware)
      .merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api));

  let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 8080))
    .await
    .unwrap();
  info!("starting http://localhost:8080");
  info!("starting swagger http://localhost:8080/swagger-ui");
  let _ = axum::serve(listener, router).await;
}
