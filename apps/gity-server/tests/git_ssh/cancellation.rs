use super::support::Fixture;
use russh::{
  ChannelMsg,
  keys::{HashAlg, PrivateKeyWithHashAlg, PublicKeyOrCertificate},
};
use std::{sync::Arc, time::Duration};
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn listener_shutdown_closes_connections_stalled_before_authentication() {
  let f = Fixture::new().await;
  let mut stalled = tokio::net::TcpStream::connect(("127.0.0.1", f.port))
    .await
    .unwrap();
  stalled
    .write_all(b"SSH-2.0-gity-stalled-test\r\n")
    .await
    .unwrap();
  f.stop().await;
}

struct KnownHost(String);
impl russh::client::Handler for KnownHost {
  type Error = russh::Error;
  async fn check_server_key(&mut self, key: &PublicKeyOrCertificate) -> Result<bool, Self::Error> {
    Ok(key.public_key().fingerprint(HashAlg::Sha256).to_string() == self.0)
  }
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn closing_one_channel_and_stopping_service_reap_native_git_without_closing_otherwise_idle_tcp()
 {
  let f = Fixture::new().await;
  let (key, _) = f.key("cancel-key", true).await;
  let host = russh::keys::load_secret_key(&f.state.settings.ssh.host_key, None).unwrap();
  let client_config = Arc::new(russh::client::Config::default());
  let mut client = russh::client::connect(
    client_config,
    ("127.0.0.1", f.port),
    KnownHost(host.public_key().fingerprint(HashAlg::Sha256).to_string()),
  )
  .await
  .unwrap();
  let key = russh::keys::load_secret_key(key, None).unwrap();
  assert!(
    client
      .authenticate_publickey("git", PrivateKeyWithHashAlg::new(Arc::new(key), None))
      .await
      .unwrap()
      .success()
  );
  let repo = f.root.path().join("repos/20.git");
  for stopping in [false, true] {
    let mut channel = client.channel_open_session().await.unwrap();
    channel
      .exec(true, "git-upload-pack 'team/demo.git'")
      .await
      .unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
      loop {
        if matches!(channel.wait().await, Some(ChannelMsg::Data { .. })) {
          break;
        }
      }
    })
    .await
    .unwrap();
    let pids = git_children(&repo);
    assert!(
      !pids.is_empty(),
      "native upload-pack must be active after ref advertisement"
    );
    if stopping {
      f.shutdown.cancel();
    } else {
      channel.close().await.unwrap();
    }
    tokio::time::timeout(Duration::from_secs(5), async {
      while pids
        .iter()
        .any(|pid| std::path::Path::new(&format!("/proc/{pid}")).exists())
      {
        tokio::time::sleep(Duration::from_millis(20)).await;
      }
    })
    .await
    .expect("cancellation must kill and reap the native process");
  }
  tokio::time::timeout(Duration::from_secs(5), f.server)
    .await
    .unwrap()
    .unwrap()
    .unwrap();
}
#[cfg(target_os = "linux")]
fn git_children(repo: &std::path::Path) -> Vec<String> {
  std::fs::read_dir("/proc")
    .unwrap()
    .filter_map(Result::ok)
    .filter_map(|entry| {
      let pid = entry.file_name().to_string_lossy().into_owned();
      if pid.parse::<u32>().is_err() {
        return None;
      }
      let cwd = std::fs::read_link(entry.path().join("cwd")).ok()?;
      (cwd == repo).then_some(pid)
    })
    .collect()
}
