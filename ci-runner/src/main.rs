use crate::cli::Args;
use bollard::Docker;
use clap::Parser;
use tracing::info;

mod cli;

async fn init() {
  dotenvy::dotenv().ok();
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_test_writer()
    .init();
}

#[tokio::main]
async fn main() {
  init().await;
  let client = Docker::connect_with_local_defaults().unwrap();
  info!("docker info {:?}", client.info().await.unwrap());
  let args = Args::parse();

  for _ in 0..args.count {
    println!("Hello {}!", args.name);
  }
}
