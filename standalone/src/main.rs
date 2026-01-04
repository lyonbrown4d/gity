use crate::app_state::{init};
use crate::configuration::cfg::Config;
use crate::rest::{build_router};
use std::net::Ipv4Addr;
use tokio::net::TcpListener;
use tracing::info;

mod app_state;
mod cli;
mod configuration;
mod rest;
mod service;
mod security;

fn init_logging() {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_test_writer()
    .init();
}

fn init_env() -> Config {
  dotenvy::dotenv().ok();
  let cfg = configuration::loader::load();
  let cfg_json = serde_json::to_string_pretty(&cfg).unwrap();
  info!("Loaded configuration:\n{}", cfg_json);
  cfg
}

#[tokio::main]
async fn main() {
  init_logging();
  let config = init_env();
  let app_state = init(config).await;

  let router = build_router(app_state);

  let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 8080))
    .await
    .unwrap();

  info!("starting http://localhost:8080");
  info!("starting swagger http://localhost:8080/swagger-ui");
  axum::serve(listener, router).await.unwrap();
}
