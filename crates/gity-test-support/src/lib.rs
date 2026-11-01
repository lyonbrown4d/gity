//! Docker-backed integration fixtures. Never attach to an existing inspection stack.
use anyhow::Result;
use testcontainers::{
  ContainerAsync, GenericImage, ImageExt,
  core::{IntoContainerPort, WaitFor},
  runners::AsyncRunner,
};

pub struct DatabaseContainer {
  _container: ContainerAsync<GenericImage>,
  dsn: String,
}
impl DatabaseContainer {
  pub fn dsn(&self) -> &str {
    &self.dsn
  }
  pub async fn postgres() -> Result<Self> {
    let container = GenericImage::new("postgres", "18")
      .with_exposed_port(5432.tcp())
      .with_wait_for(WaitFor::message_on_stderr(
        "database system is ready to accept connections",
      ))
      .with_env_var("POSTGRES_USER", "gity")
      .with_env_var("POSTGRES_PASSWORD", "gity_test_only")
      .with_env_var("POSTGRES_DB", "gity")
      .start()
      .await?;
    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(5432.tcp()).await?;
    Ok(Self {
      _container: container,
      dsn: format!("postgres://gity:gity_test_only@{host}:{port}/gity"),
    })
  }
  pub async fn mysql() -> Result<Self> {
    let container = GenericImage::new("mysql", "8.4")
      .with_exposed_port(3306.tcp())
      .with_wait_for(WaitFor::message_on_stderr("ready for connections"))
      .with_env_var("MYSQL_ROOT_PASSWORD", "gity_test_only")
      .with_env_var("MYSQL_DATABASE", "gity")
      .start()
      .await?;
    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(3306.tcp()).await?;
    Ok(Self {
      _container: container,
      dsn: format!("mysql://root:gity_test_only@{host}:{port}/gity"),
    })
  }
}
