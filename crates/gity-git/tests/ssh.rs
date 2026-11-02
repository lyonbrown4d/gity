use gity_git::{CreateFileCommitInput, GitRpc, GitService, SshPolicy};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn fixture() -> (tempfile::TempDir, GitService) {
  let directory = tempfile::tempdir().unwrap();
  let service = GitService::new(directory.path(), "git");
  service.init_bare("test.git", "main").await.unwrap();
  service
    .create_file_commit(
      "test.git",
      CreateFileCommitInput {
        branch_name: "main".into(),
        file_path: "README.md".into(),
        content: "SSH fixture".into(),
        message: "initial".into(),
        author_name: "SSH test".into(),
        author_email: "ssh@example.test".into(),
      },
    )
    .await
    .unwrap();
  (directory, service)
}
async fn advertisement(process: &mut gity_git::GitProcess) -> Vec<u8> {
  let mut stdout = process.take_stdout().unwrap();
  let mut bytes = vec![];
  loop {
    let mut prefix = [0; 4];
    tokio::time::timeout(Duration::from_secs(5), stdout.read_exact(&mut prefix))
      .await
      .expect("SSH Git must advertise before client input")
      .unwrap();
    if &prefix == b"0000" {
      break;
    }
    let length = usize::from_str_radix(std::str::from_utf8(&prefix).unwrap(), 16).unwrap();
    assert!((4..=65520).contains(&length));
    let mut packet = vec![0; length - 4];
    stdout.read_exact(&mut packet).await.unwrap();
    bytes.extend(packet);
  }
  bytes
}

#[tokio::test]
async fn ssh_upload_pack_advertises_refs_before_client_sends_data() {
  let (_directory, service) = fixture().await;
  let mut process = service
    .spawn_ssh("test.git", GitRpc::UploadPack, None, None)
    .await
    .unwrap();
  let advertised = advertisement(&mut process).await;
  assert!(String::from_utf8_lossy(&advertised).contains("refs/heads/main"));
  let mut stdin = process.take_stdin().unwrap();
  stdin.write_all(b"0000").await.unwrap();
  stdin.shutdown().await.unwrap();
  drop(stdin);
  tokio::time::timeout(Duration::from_secs(5), process.wait())
    .await
    .unwrap()
    .unwrap();
}

#[tokio::test]
async fn ssh_receive_pack_requires_policy_and_managed_hook_and_then_advertises() {
  let (directory, service) = fixture().await;
  assert!(service
    .spawn_ssh("test.git", GitRpc::ReceivePack, None, None)
    .await
    .is_err());
  let policy = SshPolicy {
    default_branch: "main".into(),
    actor_level: 30,
    rules: vec![],
  };
  assert!(service
    .spawn_ssh("test.git", GitRpc::ReceivePack, None, Some(&policy))
    .await
    .is_err());
  let executable = std::env::current_exe().unwrap();
  service
    .ensure_update_hook("test.git", &executable)
    .await
    .unwrap();
  let hook = directory.path().join("test.git/hooks/update");
  assert!(std::fs::read_to_string(&hook)
    .unwrap()
    .contains("gity-rust-protection-v1"));
  let mut process = service
    .spawn_ssh("test.git", GitRpc::ReceivePack, None, Some(&policy))
    .await
    .unwrap();
  assert!(String::from_utf8_lossy(&advertisement(&mut process).await).contains("refs/heads/main"));
  let mut stdin = process.take_stdin().unwrap();
  stdin.write_all(b"0000").await.unwrap();
  stdin.shutdown().await.unwrap();
  drop(stdin);
  tokio::time::timeout(Duration::from_secs(5), process.wait())
    .await
    .unwrap()
    .unwrap();
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(hook, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(service
      .spawn_ssh("test.git", GitRpc::ReceivePack, None, Some(&policy))
      .await
      .is_err());
  }
}

#[tokio::test]
async fn ssh_rejects_invalid_git_protocol_and_repository_traversal() {
  let (_directory, service) = fixture().await;
  for protocol in ["version=3", "version=2\nGIT_CONFIG_PARAMETERS=unsafe"] {
    assert!(service
      .spawn_ssh("test.git", GitRpc::UploadPack, Some(protocol), None)
      .await
      .is_err());
  }
  assert!(service
    .spawn_ssh("../outside.git", GitRpc::UploadPack, None, None)
    .await
    .is_err());
}

#[tokio::test]
async fn ssh_advertisement_hides_server_owned_merge_operation_references() {
  let (directory, service) = fixture().await;
  {
    let repo = git2::Repository::open_bare(directory.path().join("test.git")).unwrap();
    let tip = repo.head().unwrap().target().unwrap();
    repo
      .reference(
        "refs/gity/merge-operations/private",
        tip,
        true,
        "test private journal",
      )
      .unwrap();
  }
  let mut process = service
    .spawn_ssh("test.git", GitRpc::UploadPack, None, None)
    .await
    .unwrap();
  let refs = String::from_utf8(advertisement(&mut process).await).unwrap();
  assert!(refs.contains("refs/heads/main"));
  assert!(!refs.contains("refs/gity/"));
  let mut input = process.take_stdin().unwrap();
  input.write_all(b"0000").await.unwrap();
  input.shutdown().await.unwrap();
  drop(input);
  tokio::time::timeout(Duration::from_secs(5), process.wait())
    .await
    .unwrap()
    .unwrap();
}
