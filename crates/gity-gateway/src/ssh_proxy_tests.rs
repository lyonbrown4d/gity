use super::serve;
use std::{sync::Arc, time::Duration};
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::TcpListener,
  time::timeout,
};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn forwards_ssh_bytes_in_both_directions_and_stops_cleanly() {
  let echo_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let echo_address = echo_listener.local_addr().unwrap();
  let echo = tokio::spawn(async move {
    let (mut stream, _) = echo_listener.accept().await.unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await?;
    stream.write_all(&bytes).await
  });
  let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let proxy_address = proxy_listener.local_addr().unwrap();
  let stop = CancellationToken::new();
  let serving = tokio::spawn(serve(
    proxy_listener,
    Arc::from(echo_address.to_string()),
    1,
    Duration::from_secs(2),
    Duration::from_secs(5),
    Duration::from_secs(2),
    stop.clone(),
  ));
  let mut client = tokio::net::TcpStream::connect(proxy_address).await.unwrap();
  client.write_all(b"ssh transport").await.unwrap();
  client.shutdown().await.unwrap();
  let mut response = Vec::new();
  timeout(
    Duration::from_secs(2),
    tokio::io::copy(&mut client, &mut response),
  )
  .await
  .unwrap()
  .unwrap();
  assert_eq!(response, b"ssh transport");
  stop.cancel();
  timeout(Duration::from_secs(2), serving)
    .await
    .unwrap()
    .unwrap()
    .unwrap();
  echo.abort();
}
