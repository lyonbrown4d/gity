use gity_git::{GitService, InitialCommit, InitialFile};
use serde_json::Value;
use std::{path::Path, process::Command};

fn git(path: &Path, args: &[&str]) -> String {
  let output = Command::new("git")
    .current_dir(path)
    .args(args)
    .output()
    .unwrap();
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn initial() -> InitialCommit {
  InitialCommit {
    files: vec![
      InitialFile {
        path: "README.md".into(),
        content: "Welcome\n".into(),
      },
      InitialFile {
        path: "src/main.rs".into(),
        content: "fn main() {}\n".into(),
      },
      InitialFile {
        path: "LICENSE".into(),
        content: "MIT\n".into(),
      },
    ],
    author_name: "Gity User".into(),
    author_email: "user@example.test".into(),
  }
}

#[tokio::test]
async fn initializes_empty_or_single_root_commit_and_rejects_invalid_input() {
  let temp = tempfile::tempdir().unwrap();
  let service = GitService::new(temp.path(), "git");
  service.init_bare("empty.git", "main").await.unwrap();
  assert_eq!(
    service
      .list_commits("empty.git", "", "main", 10)
      .await
      .unwrap(),
    Value::Array(vec![])
  );

  service
    .init_bare_with_commit("seed.git", "main", Some(initial()))
    .await
    .unwrap();
  let commits = service
    .list_commits("seed.git", "", "main", 10)
    .await
    .unwrap();
  assert_eq!(commits.as_array().unwrap().len(), 1);
  assert_eq!(
    git(
      &temp.path().join("seed.git"),
      &["rev-list", "--count", "main"]
    ),
    "1"
  );
  assert_eq!(
    service
      .get_blob("seed.git", "", "main", "src/main.rs")
      .await
      .unwrap()["content"],
    "fn main() {}\n"
  );

  assert!(service
    .init_bare_with_commit("invalid.git", "bad branch", Some(initial()))
    .await
    .is_err());
  let mut duplicate = initial();
  duplicate.files.push(duplicate.files[0].clone());
  assert!(service
    .init_bare_with_commit("duplicate.git", "main", Some(duplicate))
    .await
    .is_err());
  let too_deep = InitialCommit {
    files: vec![InitialFile {
      path: format!("{}/file.txt", vec!["d"; 64].join("/")),
      content: "x".into(),
    }],
    ..initial()
  };
  assert!(service
    .init_bare_with_commit("too-deep.git", "main", Some(too_deep))
    .await
    .is_err());
  assert!(!temp.path().join("too-deep.git").exists());
  let invalid_signature = InitialCommit {
    author_name: "invalid\nname".into(),
    ..initial()
  };
  assert!(service
    .init_bare_with_commit("invalid-signature.git", "main", Some(invalid_signature))
    .await
    .is_err());
  assert!(!temp.path().join("invalid-signature.git").exists());
  assert!(service
    .init_bare_with_commit("seed.git", "main", Some(initial()))
    .await
    .is_err());
  assert_eq!(
    git(
      &temp.path().join("seed.git"),
      &["rev-list", "--count", "main"]
    ),
    "1"
  );
}
