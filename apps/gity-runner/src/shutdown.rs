use std::sync::OnceLock;
use tokio_util::sync::CancellationToken;
fn token() -> &'static CancellationToken {
  static TOKEN: OnceLock<CancellationToken> = OnceLock::new();
  TOKEN.get_or_init(CancellationToken::new)
}
pub fn install() {
  tokio::spawn(async {
    #[cfg(unix)]
    {
      let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("install Runner SIGTERM");
      tokio::select! {_=term.recv()=>{},_=tokio::signal::ctrl_c()=>{}}
    }
    #[cfg(not(unix))]
    {
      let _ = tokio::signal::ctrl_c().await;
    }
    token().cancel();
  });
}
pub async fn cancelled() {
  token().cancelled().await;
}
pub fn is_cancelled() -> bool {
  token().is_cancelled()
}
