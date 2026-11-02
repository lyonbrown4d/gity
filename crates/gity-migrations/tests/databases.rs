mod support;

#[tokio::test]
async fn sqlite_latest_schema_contract() {
  support::assert_schema(support::connect("sqlite::memory:").await).await;
}

#[tokio::test]
#[ignore = "requires Docker; starts an isolated PostgreSQL 18 Testcontainer"]
async fn postgres_latest_schema_contract() {
  let container = gity_test_support::DatabaseContainer::postgres()
    .await
    .unwrap();
  support::assert_schema(support::connect(container.dsn()).await).await;
}

#[tokio::test]
#[ignore = "requires Docker; starts an isolated MySQL 8.4 Testcontainer"]
async fn mysql_latest_schema_contract() {
  let container = gity_test_support::DatabaseContainer::mysql().await.unwrap();
  support::assert_schema(support::connect(container.dsn()).await).await;
}
