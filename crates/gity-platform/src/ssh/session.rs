use super::connection::Connection;
use crate::AppState;
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpStream, sync::Notify};
use tokio_util::sync::CancellationToken;

struct SocketGuard {
  socket: std::net::TcpStream,
  disconnect: CancellationToken,
}
impl SocketGuard {
  fn close(&self) {
    self.disconnect.cancel();
    let _ = self.socket.shutdown(std::net::Shutdown::Both);
  }
}
impl Drop for SocketGuard {
  fn drop(&mut self) {
    self.close();
  }
}

pub(super) async fn run(
  state: AppState,
  config: Arc<russh::server::Config>,
  stream: TcpStream,
  shutdown: CancellationToken,
) -> anyhow::Result<()> {
  let socket = stream.into_std()?;
  let disconnect = CancellationToken::new();
  let guard = SocketGuard {
    socket: socket.try_clone()?,
    disconnect: disconnect.clone(),
  };
  let stream = TcpStream::from_std(socket)?;
  let authenticated = Arc::new(Notify::new());
  let handler = Connection::new(state, authenticated.clone(), disconnect);
  let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
  let mut session = tokio::select! {
      _=shutdown.cancelled()=>return Ok(()),
      result=tokio::time::timeout_at(deadline,russh::server::run_stream(config,stream,handler))=>result??,
  };
  tokio::select! {
      result=&mut session=>return result,
      _=shutdown.cancelled()=>{},
      result=tokio::time::timeout_at(deadline,authenticated.notified())=>{
          if result.is_ok() {
              tokio::select! {
                  result=&mut session=>return result,
                  _=shutdown.cancelled()=>{},
              }
          }
      }
  }
  guard.close();
  // Await the library session so shutdown does not detach its handler/tasks.
  tokio::time::timeout(Duration::from_secs(15), &mut session).await??;
  Ok(())
}
