use std::{io, sync::Arc, time::Duration};
use tokio::{
  net::{TcpListener, TcpStream},
  task::JoinSet,
  time::timeout,
};
use tokio_util::sync::CancellationToken;

pub(super) async fn serve(
  listener: TcpListener,
  target: Arc<str>,
  max_sessions: usize,
  connect_timeout: Duration,
  session_timeout: Duration,
  shutdown_timeout: Duration,
  stop: CancellationToken,
) -> io::Result<()> {
  let slots = Arc::new(tokio::sync::Semaphore::new(max_sessions));
  let mut sessions = JoinSet::new();
  loop {
    tokio::select! {
      _ = stop.cancelled() => break,
      incoming = listener.accept() => {
        let (client, _) = incoming?;
        let Ok(slot) = slots.clone().try_acquire_owned() else {
          drop(client);
          continue;
        };
        let target = target.clone();
        let stop_session = stop.child_token();
        sessions.spawn(async move {
          if let Err(error) = proxy_session(
            client,
            target,
            slot,
            connect_timeout,
            session_timeout,
            stop_session,
          )
          .await
          {
            tracing::debug!(%error, "gateway SSH session ended");
          }
        });
      }
      Some(_) = sessions.join_next(), if !sessions.is_empty() => {}
    }
  }
  stop.cancel();
  if timeout(shutdown_timeout, async {
    while sessions.join_next().await.is_some() {}
  })
  .await
  .is_err()
  {
    sessions.abort_all();
    while sessions.join_next().await.is_some() {}
  }
  Ok(())
}

async fn proxy_session(
  mut client: TcpStream,
  target: Arc<str>,
  _slot: tokio::sync::OwnedSemaphorePermit,
  connect_timeout: Duration,
  session_timeout: Duration,
  stop: CancellationToken,
) -> io::Result<()> {
  client.set_nodelay(true)?;
  let connect = TcpStream::connect(target.as_ref());
  let mut upstream = timeout(connect_timeout, connect)
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "SSH upstream connect timed out"))??;
  upstream.set_nodelay(true)?;
  tokio::select! {
    _ = stop.cancelled() => Ok(()),
    result = timeout(session_timeout, tokio::io::copy_bidirectional(&mut client, &mut upstream)) => {
      match result {
        Ok(result) => result.map(|_| ()),
        Err(_) => Err(io::Error::new(io::ErrorKind::TimedOut, "SSH session timed out")),
      }
    }
  }
}

#[cfg(test)]
#[path = "ssh_proxy_tests.rs"]
mod tests;
