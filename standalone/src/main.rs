use std::net::Ipv4Addr;
use http::app_state::AppState;
use std::sync::Arc;
use sadi::Injector;
use tokio::net::TcpListener;
use tracing::log::info;
use crate::configuration::cfg::Config;
use crate::http::build_router;

mod cli;
mod configuration;
mod http;
mod service;
mod security;
pub mod bootstrap;
mod repository;

#[tokio::main]
async fn main() {
  let app = bootstrap::bootstrap().unwrap();
  let injector: Arc<Injector> = Arc::new((*app.injector()).clone());
  // let config = bootstrap::bootstrap();
  let app_state = AppState{
    injector
  };
  let router = build_router(app_state);
  //
  let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 8080))
    .await
    .unwrap();
  info!("starting http://localhost:8080");
  info!("starting swagger http://localhost:8080/swagger-ui");
  axum::serve(listener, router).await.unwrap();
}
