//! Run existing API contracts against isolated database containers, never a kept stack.
use gity_test_support::DatabaseContainer;
use std::{path::Path, process::Command};

async fn run_contracts(database: &DatabaseContainer) {
  tokio::time::timeout(std::time::Duration::from_secs(60), async {
    loop {
      if let Ok(connection) = sea_orm::Database::connect(database.dsn()).await
        && connection.ping().await.is_ok()
      {
        connection.close().await.unwrap();
        break;
      }
      tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
  })
  .await
  .expect("isolated database authentication readiness");
  let root = Path::new(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .unwrap()
    .parent()
    .unwrap();
  let result = Command::new(env!("CARGO"))
    .current_dir(root)
    .args([
      "test",
      "-p",
      "gity-platform",
      "-p",
      "gity-server",
      "--locked",
      "--",
      "--test-threads=1",
    ])
    .env("GITY_TEST_DATABASE_URL", database.dsn())
    .status()
    .expect("run existing database API regression suite");
  assert!(result.success(), "database API regression failed");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires Docker; runs existing platform/server contracts in isolated PostgreSQL"]
async fn postgres_business_regression() {
  run_contracts(&DatabaseContainer::postgres().await.unwrap()).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires Docker; runs existing platform/server contracts in isolated MySQL"]
async fn mysql_business_regression() {
  run_contracts(&DatabaseContainer::mysql().await.unwrap()).await;
}
