use gity_git::{CreateFileCommitInput, GitService, SearchParams};
use gity_search::{Limits, SearchIndex};
fn file(path: &str, content: &str) -> CreateFileCommitInput {
  CreateFileCommitInput {
    expected_head_sha: None,
    update_existing: false,
    branch_name: "main".into(),
    file_path: path.into(),
    content: content.into(),
    message: format!("Add {path}"),
    author_name: "Tester".into(),
    author_email: "test@example.test".into(),
  }
}
fn query(value: &str) -> SearchParams {
  SearchParams {
    query: value.into(),
    ..Default::default()
  }
}
async fn fixture() -> (tempfile::TempDir, GitService, SearchIndex) {
  let directory = tempfile::tempdir().unwrap();
  let root = directory.path().join("repos");
  std::fs::create_dir(&root).unwrap();
  let git = GitService::new(&root, "git");
  git.init_bare("demo.git", "main").await.unwrap();
  let index = SearchIndex::new(
    directory.path().join("index"),
    git.clone(),
    Limits::default(),
  )
  .unwrap();
  (directory, git, index)
}
#[tokio::test]
async fn persisted_candidates_match_live_lines_and_stale_revisions_fall_back() {
  let (directory, git, index) = fixture().await;
  git
    .create_file_commit(
      "demo.git",
      file(
        "README.md",
        "你好 needle here\nſtone and Kelvin\nx+y abc xyz\n",
      ),
    )
    .await
    .unwrap();
  git
    .create_file_commit(
      "demo.git",
      file("src/main.rs", "// Needle in code\nfn my_function() {}\n"),
    )
    .await
    .unwrap();
  let status = index.refresh(7, "demo.git", "main").await.unwrap();
  assert_eq!(status.indexed_files, 2);
  assert!(!status.truncated);
  assert!(
    index
      .refresh(7, "demo.git", "main")
      .await
      .unwrap()
      .unchanged
  );
  let reopened = SearchIndex::new(
    directory.path().join("index"),
    git.clone(),
    Limits::default(),
  )
  .unwrap();
  for term in ["needle", "STONE", "Kelvin", "my_fun", "abc xyz", "x+y"] {
    let indexed = reopened
      .query(7, "demo.git", "", "main", query(term))
      .await
      .unwrap()
      .unwrap();
    let live = git
      .search("demo.git", "", "main", query(term))
      .await
      .unwrap();
    assert_eq!(indexed, live, "{term}");
  }
  let scoped = SearchParams {
    query: "needle".into(),
    path: "src".into(),
    match_case: true,
    ..Default::default()
  };
  assert_eq!(
    index
      .query(7, "demo.git", "main", "main", scoped.clone())
      .await
      .unwrap()
      .unwrap(),
    git
      .search("demo.git", "main", "main", scoped)
      .await
      .unwrap()
  );
  for input in [
    query("你好"),
    query("aa"),
    SearchParams {
      regex: true,
      ..query("needle")
    },
  ] {
    assert!(
      index
        .query(7, "demo.git", "", "main", input)
        .await
        .unwrap()
        .is_none()
    );
  }
  assert!(
    index
      .query(7, "demo.git", "feature", "main", query("needle"))
      .await
      .unwrap()
      .is_none()
  );
  git
    .create_file_commit("demo.git", file("new.txt", "needle new commit"))
    .await
    .unwrap();
  assert!(
    index
      .query(7, "demo.git", "", "main", query("needle"))
      .await
      .unwrap()
      .is_none()
  );
  index.refresh(7, "demo.git", "main").await.unwrap();
  assert_eq!(
    index
      .query(7, "demo.git", "", "main", query("needle"))
      .await
      .unwrap()
      .unwrap()
      .as_array()
      .unwrap()
      .len(),
    3
  );
  index.delete(7).await.unwrap();
  assert!(
    index
      .query(7, "demo.git", "", "main", query("needle"))
      .await
      .unwrap()
      .is_none()
  );
}
#[tokio::test]
async fn incomplete_and_busy_indexes_never_return_authoritative_partial_results() {
  let (directory, git, _) = fixture().await;
  git
    .create_file_commit("demo.git", file("one", "needle one"))
    .await
    .unwrap();
  git
    .create_file_commit("demo.git", file("two", "needle two"))
    .await
    .unwrap();
  let root = directory.path().join("index");
  let index = SearchIndex::new(
    &root,
    git.clone(),
    Limits {
      max_files: 1,
      ..Default::default()
    },
  )
  .unwrap();
  assert!(
    index
      .refresh(7, "demo.git", "main")
      .await
      .unwrap()
      .truncated
  );
  assert!(
    index
      .query(7, "demo.git", "", "main", query("needle"))
      .await
      .unwrap()
      .is_none()
  );
  let lock = std::fs::OpenOptions::new()
    .read(true)
    .write(true)
    .open(root.join("7.lock"))
    .unwrap();
  fs2::FileExt::lock_exclusive(&lock).unwrap();
  assert!(
    index
      .query(7, "demo.git", "", "main", query("needle"))
      .await
      .unwrap()
      .is_none()
  );
  assert!(index.refresh(7, "demo.git", "main").await.is_err());
  drop(lock);
  #[cfg(unix)]
  {
    std::os::unix::fs::symlink(directory.path(), root.join("8")).unwrap();
    assert!(index.refresh(8, "demo.git", "main").await.is_err());
  }
  assert!(index.delete(-1).await.is_err());
}
#[tokio::test]
async fn empty_repositories_have_no_indexable_commit() {
  let (_directory, _git, index) = fixture().await;
  let status = index.refresh(7, "demo.git", "main").await.unwrap();
  assert_eq!(status.indexed_files, 0);
  assert!(status.revision.is_empty());
}
