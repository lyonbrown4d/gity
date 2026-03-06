use crate::http::build_router;
use http::app_state::AppState;
use std::net::{IpAddr, Ipv4Addr};
use tokio::net::TcpListener;
use tracing::log::info;

pub mod bootstrap;
mod cli;
mod configuration;
mod http;
mod jobs;
mod security;
mod service;

#[tokio::main]
async fn main() {
  let app_state: AppState = bootstrap::bootstrap().await.unwrap();
  let bind_ip = if app_state.config.server.publish.unwrap_or(false) {
    IpAddr::V4(Ipv4Addr::UNSPECIFIED)
  } else {
    IpAddr::V4(Ipv4Addr::LOCALHOST)
  };
  let bind_port = app_state.config.server.port as u16;
  let router = build_router(app_state);

  let listener = TcpListener::bind((bind_ip, bind_port)).await.unwrap();
  info!("starting http://{}:{}", bind_ip, bind_port);
  info!(
    "starting swagger http://{}:{}/swagger-ui",
    bind_ip, bind_port
  );
  axum::serve(listener, router).await.unwrap();
}
