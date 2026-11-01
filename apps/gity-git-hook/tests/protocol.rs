use gity_git::{CreateFileCommitInput, GitRpc, GitService};
use std::{collections::BTreeMap, path::Path, process::Command};
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}
async fn fixture() -> (TempDir, GitService) {
    let dir = tempfile::tempdir().unwrap();
    let svc = GitService::new(dir.path(), "git");
    svc.init_bare("team/demo.git", "main").await.unwrap();
    (dir, svc)
}
fn file(branch: &str, path: &str, content: &str) -> CreateFileCommitInput {
    CreateFileCommitInput {
        branch_name: branch.into(),
        file_path: path.into(),
        content: content.into(),
        message: format!("Add {path}"),
        author_name: "Tester".into(),
        author_email: "test@example.test".into(),
    }
}
#[tokio::test]
async fn protocol_advertisement_fetch_and_protected_push() {
    let (dir, svc) = fixture().await;
    svc.create_file_commit("team/demo.git", file("main", "README.md", "base"))
        .await
        .unwrap();
    let repo = dir.path().join("team/demo.git");
    svc.ensure_update_hook("team/demo.git", env!("CARGO_BIN_EXE_gity-git-hook"))
        .await
        .unwrap();
    let process = svc
        .spawn_rpc("team/demo.git", GitRpc::UploadPack, true, BTreeMap::new())
        .await
        .unwrap();
    let completed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let completion = completed.clone();
    let mut stdout = process
        .into_reader_with_completion(move || async move {
            completion.store(true, std::sync::atomic::Ordering::Release);
        })
        .unwrap();
    let mut output = Vec::new();
    stdout.read_to_end(&mut output).await.unwrap();
    assert!(String::from_utf8_lossy(&output).contains("refs/heads/main"));
    assert!(completed.load(std::sync::atomic::Ordering::Acquire));
    let id = svc
        .resolve_revision("team/demo.git", "main", "")
        .await
        .unwrap();
    let mut request = Vec::new();
    let want = format!("want {id}\n");
    request.extend_from_slice(format!("{:04x}", want.len() + 4).as_bytes());
    request.extend_from_slice(want.as_bytes());
    request.extend_from_slice(b"00000009done\n");
    let mut process = svc
        .spawn_rpc("team/demo.git", GitRpc::UploadPack, false, BTreeMap::new())
        .await
        .unwrap();
    let mut stdin = process.take_stdin().unwrap();
    stdin.write_all(&request).await.unwrap();
    stdin.shutdown().await.unwrap();
    drop(stdin);
    let mut stdout = process.take_stdout().unwrap();
    let mut output = Vec::new();
    stdout.read_to_end(&mut output).await.unwrap();
    process.wait().await.unwrap();
    assert!(output.windows(4).any(|v| v == b"PACK"));
    // Exercise the actual update hook through native Git push.
    let work = dir.path().join("clone");
    git(
        dir.path(),
        &["clone", repo.to_str().unwrap(), work.to_str().unwrap()],
    );
    git(&work, &["config", "user.name", "Tester"]);
    git(&work, &["config", "user.email", "test@example.test"]);
    std::fs::write(work.join("next"), "next").unwrap();
    git(&work, &["add", "."]);
    git(&work, &["commit", "-m", "next"]);
    git(&work, &["push", "origin", "main"]);
    git(&work, &["reset", "--hard", &id]);
    let failed = Command::new("git")
        .current_dir(&work)
        .env("GITY_DENY_FORCE_PUSH_REFS", "refs/heads/main")
        .args(["push", "--force", "origin", "main"])
        .output()
        .unwrap();
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("force push is not allowed"));
}
