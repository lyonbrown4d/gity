use gity_git::{CreateFileCommitInput, GitService};
use std::{path::Path, process::Command, sync::Arc};

fn input(path: &str, head: Option<&str>, update: bool) -> CreateFileCommitInput {
  CreateFileCommitInput {
    branch_name: "main".into(),
    file_path: path.into(),
    content: "new content".into(),
    message: "Update file".into(),
    expected_head_sha: head.map(str::to_owned),
    update_existing: update,
    author_name: "Owner".into(),
    author_email: "owner@test.local".into(),
  }
}

#[tokio::test]
async fn file_updates_require_opt_in_expected_head_and_existing_blob() {
  let root = tempfile::tempdir().unwrap();
  let service = GitService::new(root.path(), "git");
  service.init_bare("repo.git", "main").await.unwrap();
  assert!(service
    .create_file_commit("repo.git", input("file", Some(&"1".repeat(40)), false))
    .await
    .unwrap_err()
    .to_string()
    .contains("head changed"));
  let initial = service
    .create_file_commit("repo.git", input("file", Some(""), false))
    .await
    .unwrap();
  assert!(service
    .create_file_commit("repo.git", input("file", None, false))
    .await
    .unwrap_err()
    .to_string()
    .contains("already exists"));
  assert!(service
    .create_file_commit("repo.git", input("file", None, true))
    .await
    .unwrap_err()
    .to_string()
    .contains("requires expected_head_sha"));
  assert!(service
    .create_file_commit("repo.git", input("file", Some(""), true))
    .await
    .is_err());
  let mut update = input("file", Some(&initial), true);
  update.content = "saved".into();
  let updated = service
    .create_file_commit("repo.git", update)
    .await
    .unwrap();
  assert_ne!(initial, updated);
  assert!(service
    .create_file_commit("repo.git", input("file", Some(&initial), true))
    .await
    .unwrap_err()
    .to_string()
    .contains("head changed"));
  assert!(service
    .create_file_commit("repo.git", input("missing", Some(&updated), true))
    .await
    .unwrap_err()
    .to_string()
    .contains("not found"));
  let tree = service
    .create_file_commit("repo.git", input("dir/file", Some(&updated), false))
    .await
    .unwrap();
  assert!(service
    .create_file_commit("repo.git", input("dir", Some(&tree), true))
    .await
    .is_err());
  assert!(service
    .create_file_commit("repo.git", input("file/nested", Some(&tree), false))
    .await
    .is_err());
  assert_eq!(
    service
      .resolve_revision("repo.git", "main", "")
      .await
      .unwrap(),
    tree
  );
  let repository = git2::Repository::open_bare(root.path().join("repo.git")).unwrap();
  let commit = repository
    .find_commit(git2::Oid::from_str(&tree).unwrap())
    .unwrap();
  let blob = commit
    .tree()
    .unwrap()
    .get_path(Path::new("file"))
    .unwrap()
    .to_object(&repository)
    .unwrap();
  assert_eq!(blob.as_blob().unwrap().content(), b"saved");
}

fn git(dir: &Path, args: &[&str]) {
  let result = Command::new("git")
    .current_dir(dir)
    .args(args)
    .output()
    .unwrap();
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn expected_head_cas_races_safely_with_native_git_push() {
  let root = tempfile::tempdir().unwrap();
  let service = GitService::new(root.path(), "git");
  service.init_bare("repo.git", "main").await.unwrap();
  let initial = service
    .create_file_commit("repo.git", input("file", Some(""), false))
    .await
    .unwrap();
  git(root.path(), &["clone", "repo.git", "work"]);
  let work = root.path().join("work");
  git(&work, &["config", "user.name", "Native"]);
  git(&work, &["config", "user.email", "native@test.local"]);
  std::fs::write(work.join("native"), "native").unwrap();
  git(&work, &["add", "native"]);
  git(&work, &["commit", "-m", "Native push"]);
  let barrier = Arc::new(tokio::sync::Barrier::new(2));
  let ready = barrier.clone();
  let push = tokio::spawn(async move {
    ready.wait().await;
    tokio::task::spawn_blocking(move || {
      Command::new("git")
        .current_dir(work)
        .args(["push", "origin", "main"])
        .output()
        .unwrap()
        .status
        .success()
    })
    .await
    .unwrap()
  });
  barrier.wait().await;
  let result = service
    .create_file_commit("repo.git", input("file", Some(&initial), true))
    .await;
  let pushed = push.await.unwrap();
  assert_ne!(
    result.is_ok(),
    pushed,
    "one writer must fail rather than lose a successful commit"
  );
  let repository = git2::Repository::open_bare(root.path().join("repo.git")).unwrap();
  let commit = repository
    .find_reference("refs/heads/main")
    .unwrap()
    .peel_to_commit()
    .unwrap();
  assert_eq!(commit.parent_id(0).unwrap().to_string(), initial);
  assert_eq!(commit.tree().unwrap().get_name("native").is_some(), pushed);
  if let Ok(sha) = result {
    assert_eq!(commit.id().to_string(), sha);
  }
}
