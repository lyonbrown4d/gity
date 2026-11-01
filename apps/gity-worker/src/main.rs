#[tokio::main]
async fn main() -> anyhow::Result<()> {
  gity_platform::application::run(gity_platform::application::ApplicationRole::Worker).await
}
