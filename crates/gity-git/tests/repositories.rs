use gity_git::{CreateFileCommitInput, GitService, MergeBranchesInput, SearchParams};
use serde_json::Value;
use std::{path::Path, process::Command};
use tempfile::TempDir;

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
    expected_head_sha: None,
    update_existing: false,
    branch_name: branch.into(),
    file_path: path.into(),
    content: content.into(),
    message: format!("Add {path}"),
    author_name: "Tester".into(),
    author_email: "test@example.test".into(),
  }
}
fn merge(source: &str) -> MergeBranchesInput {
  MergeBranchesInput {
    target_branch: "main".into(),
    source_branch: source.into(),
    expected_source_sha: None,
    message: "Merge feature".into(),
    author_name: "Tester".into(),
    author_email: "test@example.test".into(),
  }
}
fn array(v: &Value) -> &Vec<Value> {
  v.as_array().unwrap()
}

#[tokio::test]
async fn bare_repository_reads_mutations_and_archive() {
  let (dir, svc) = fixture().await;
  assert!(array(
    &svc
      .list_tree("team/demo.git", "", "main", "")
      .await
      .unwrap()
  )
  .is_empty());
  svc
    .create_file_commit(
      "team/demo.git",
      file("main", "README.md", "你好 Needle\nhello\n"),
    )
    .await
    .unwrap();
  svc
    .create_file_commit(
      "team/demo.git",
      file("main", "src/lib.rs", "pub fn hello() {}\n"),
    )
    .await
    .unwrap();
  let tree = svc
    .list_tree("team/demo.git", "", "main", "")
    .await
    .unwrap();
  assert_eq!(tree[0]["type"], "tree");
  assert_eq!(tree[1]["name"], "README.md");
  assert_eq!(
    svc.get_readme("team/demo.git", "", "main").await.unwrap()["content"],
    "你好 Needle\nhello\n"
  );
  assert_eq!(
    array(
      &svc
        .list_commits("team/demo.git", "", "main", 10)
        .await
        .unwrap()
    )
    .len(),
    2
  );
  svc
    .create_branch("team/demo.git", "feature", "main")
    .await
    .unwrap();
  svc
    .create_file_commit("team/demo.git", file("feature", "new.txt", "feature"))
    .await
    .unwrap();
  assert!(svc
    .diff_branches("team/demo.git", "main", "feature")
    .await
    .unwrap()
    .contains("+feature"));
  let before = svc
    .resolve_revision("team/demo.git", "main", "")
    .await
    .unwrap();
  let mut stale = merge("feature");
  stale.expected_source_sha = Some(before.clone());
  assert!(svc
    .merge_branches("team/demo.git", stale)
    .await
    .unwrap_err()
    .to_string()
    .contains("source changed"));
  assert_eq!(
    svc
      .resolve_revision("team/demo.git", "main", "")
      .await
      .unwrap(),
    before
  );
  svc
    .merge_branches("team/demo.git", merge("feature"))
    .await
    .unwrap();
  let path = dir.path().join("team/demo.git");
  assert_eq!(
    git(&path, &["rev-list", "--parents", "-n", "1", "main"])
      .split_whitespace()
      .count(),
    3
  );
  svc.create_tag("team/demo.git", "v1", "main").await.unwrap();
  assert_eq!(
    svc.list_tags("team/demo.git").await.unwrap()[0]["name"],
    "v1"
  );
  let archive = svc.archive("team/demo.git", "v1").await.unwrap();
  assert_eq!(&archive[..2], b"PK");
  svc.delete_tag("team/demo.git", "v1").await.unwrap();
  svc.delete_branch("team/demo.git", "feature").await.unwrap();
  assert_eq!(
    array(&svc.list_branches("team/demo.git", "main").await.unwrap()).len(),
    1
  );
  let found = svc
    .search(
      "team/demo.git",
      "",
      "main",
      SearchParams {
        query: "needle".into(),
        ..Default::default()
      },
    )
    .await
    .unwrap();
  assert_eq!(found[0]["column"], 4);
  assert_eq!(found[0]["match_length"], 6);
  assert_eq!(
    svc
      .analyze_languages("team/demo.git", "", "main")
      .await
      .unwrap()["languages"][0]["language"],
    "Rust"
  );
}

#[tokio::test]
async fn traversal_duplicates_and_merge_conflicts_preserve_refs() {
  let (dir, svc) = fixture().await;
  for path in [
    "../escape.git",
    "/tmp/escape.git",
    "x/../../escape.git",
    "x\\escape.git",
  ] {
    assert!(svc.init_bare(path, "main").await.is_err());
  }
  #[cfg(unix)]
  {
    std::os::unix::fs::symlink("/tmp", dir.path().join("outside")).unwrap();
    assert!(svc.init_bare("outside/escape.git", "main").await.is_err());
  }
  svc
    .create_file_commit("team/demo.git", file("main", "README.md", "base"))
    .await
    .unwrap();
  svc
    .create_branch("team/demo.git", "feature", "main")
    .await
    .unwrap();
  // Distinct additions at the same path produce an add/add merge conflict.
  svc
    .create_file_commit("team/demo.git", file("main", "clash", "one"))
    .await
    .unwrap();
  svc
    .create_file_commit("team/demo.git", file("feature", "clash", "two"))
    .await
    .unwrap();
  let before = svc
    .resolve_revision("team/demo.git", "main", "")
    .await
    .unwrap();
  assert!(svc
    .merge_branches("team/demo.git", merge("feature"))
    .await
    .unwrap_err()
    .to_string()
    .contains("merge conflict"));
  assert_eq!(
    svc
      .resolve_revision("team/demo.git", "main", "")
      .await
      .unwrap(),
    before
  );
  assert!(svc
    .create_file_commit("team/demo.git", file("main", "README.md", "replace"))
    .await
    .is_err());
  assert!(svc
    .create_file_commit("team/demo.git", file("main", "../escape", "unsafe"))
    .await
    .is_err());
  assert!(svc
    .create_branch("team/demo.git", "-bad", "main")
    .await
    .is_err());
  assert!(svc
    .archive("team/demo.git", "--output=/tmp/escape")
    .await
    .is_err());
}
