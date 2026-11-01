//! FluxDI lifecycle adapters for external Gity runtime components.
mod angos;

pub use angos::{AngosConfig, AngosModule, AngosProcess};
pub use standalone::StandaloneModule;
mod standalone;

/// Common graceful termination signal for independently deployable components.
pub async fn shutdown_signal() {
  #[cfg(unix)]
  {
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
      .expect("install SIGTERM handler");
    tokio::select! { _=tokio::signal::ctrl_c()=>{}, _=terminate.recv()=>{} }
  }
  #[cfg(not(unix))]
  {
    let _ = tokio::signal::ctrl_c().await;
  }
}
