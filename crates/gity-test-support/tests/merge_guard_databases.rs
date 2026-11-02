//! Focused cross-process database locking contracts; no retained stack is used.
use gity_test_support::DatabaseContainer;
use std::{path::Path, process::Command, time::Duration};

async fn contracts(database: DatabaseContainer) {
  tokio::time::timeout(Duration::from_secs(60), async {
    loop {
      if let Ok(connection) = sea_orm::Database::connect(database.dsn()).await
        && connection.ping().await.is_ok()
      {
        connection.close().await.unwrap();
        break;
      }
      tokio::time::sleep(Duration::from_millis(250)).await;
    }
  })
  .await
  .expect("isolated database readiness");
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
      "--lib",
      "--locked",
      "features::merge_events",
      "--",
      "--test-threads=1",
      "--nocapture",
    ])
    .env("GITY_TEST_DATABASE_URL", database.dsn())
    .status()
    .expect("run merge journal contracts in a separate process");
  assert!(result.success(), "merge journal database contracts failed");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires Docker; isolated PostgreSQL merge journal concurrency contracts"]
async fn postgres_merge_guard() {
  contracts(DatabaseContainer::postgres().await.unwrap()).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires Docker; isolated MySQL merge journal concurrency contracts"]
async fn mysql_merge_guard() {
  contracts(DatabaseContainer::mysql().await.unwrap()).await;
}
