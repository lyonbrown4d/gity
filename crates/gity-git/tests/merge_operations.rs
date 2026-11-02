use gity_git::{CreateFileCommitInput, GitService, MergeOperationInput, MergeOperationOutcome};

fn file(branch: &str, name: &str) -> CreateFileCommitInput {
  CreateFileCommitInput {
    branch_name: branch.into(),
    file_path: name.into(),
    content: name.into(),
    message: name.into(),
    author_name: "Tester".into(),
    author_email: "test@example.test".into(),
  }
}
async fn fixture() -> (tempfile::TempDir, GitService, MergeOperationInput) {
  let dir = tempfile::tempdir().unwrap();
  let git = GitService::new(dir.path(), "git");
  git.init_bare("20.git", "main").await.unwrap();
  let target = git
    .create_file_commit("20.git", file("main", "base"))
    .await
    .unwrap();
  git
    .create_branch("20.git", "feature", "main")
    .await
    .unwrap();
  let source = git
    .create_file_commit("20.git", file("feature", "feature"))
    .await
    .unwrap();
  let input = MergeOperationInput {
    operation_key: "019a3955-1000-7000-8000-000000000001".into(),
    actor_user_id: 1,
    source_branch: "feature".into(),
    target_branch: "main".into(),
    source_sha: source,
    target_sha: target,
    message: "Merge feature".into(),
    author_name: "Tester".into(),
    author_email: "test@example.test".into(),
    timestamp: 1_800_000_000,
  };
  (dir, git, input)
}
fn sha(outcome: &MergeOperationOutcome) -> &str {
  match outcome {
    MergeOperationOutcome::Applied { commit_sha } => commit_sha,
    _ => panic!("expected merge"),
  }
}
#[tokio::test]
async fn lost_response_restart_reconciles_exact_commit_after_target_advances() {
  let (dir, git, input) = fixture().await;
  let result = git.merge_operation("20.git", input.clone()).await.unwrap();
  let later = git
    .create_file_commit("20.git", file("main", "later"))
    .await
    .unwrap();
  let restarted = GitService::new(dir.path(), "git");
  assert_ne!(sha(&result), later);
  assert_eq!(
    restarted
      .merge_operation_outcome("20.git", input.clone())
      .await
      .unwrap(),
    Some(result.clone())
  );
  assert_eq!(
    restarted
      .merge_operation("20.git", input.clone())
      .await
      .unwrap(),
    result
  );
  assert_eq!(
    restarted
      .resolve_revision("20.git", "main", "")
      .await
      .unwrap(),
    later
  );
  let snapshot = restarted.reference_snapshot("20.git").await.unwrap();
  assert!(snapshot.refs.contains_key(&format!(
    "refs/gity/merge-operations/{}",
    input.operation_key
  )));
  let mut changed = input;
  changed.message = "different intent".into();
  assert!(restarted.merge_operation("20.git", changed).await.is_err());
}
#[tokio::test]
async fn both_changed_pins_persist_rejection_across_retries() {
  for branch in ["main", "feature"] {
    let (_dir, git, input) = fixture().await;
    git
      .create_file_commit("20.git", file(branch, "later"))
      .await
      .unwrap();
    let outcome = git.merge_operation("20.git", input.clone()).await.unwrap();
    assert!(matches!(outcome, MergeOperationOutcome::Rejected { .. }));
    assert_eq!(git.merge_operation("20.git", input).await.unwrap(), outcome);
    let (_dir, git, input) = fixture().await;
    git.delete_branch("20.git", branch).await.unwrap();
    let outcome = git.merge_operation("20.git", input.clone()).await.unwrap();
    assert!(matches!(outcome, MergeOperationOutcome::Rejected { .. }));
    assert_eq!(git.merge_operation("20.git", input).await.unwrap(), outcome);
  }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn independent_instances_share_one_immutable_outcome() {
  let (dir, git, input) = fixture().await;
  let other = GitService::new(dir.path(), "git");
  let (a, b) = tokio::join!(
    git.merge_operation("20.git", input.clone()),
    other.merge_operation("20.git", input.clone())
  );
  assert!(a.is_ok() || b.is_ok());
  let expected = a.or(b).unwrap();
  assert_eq!(
    git.merge_operation("20.git", input.clone()).await.unwrap(),
    expected
  );
  assert_eq!(
    other.merge_operation("20.git", input).await.unwrap(),
    expected
  );
}
#[tokio::test]
async fn prepare_crash_resumes_same_candidate_and_abort_fences_stale_retry() {
  let (dir, git, input) = fixture().await;
  let applied = git.merge_operation("20.git", input.clone()).await.unwrap();
  let repo = git2::Repository::open_bare(dir.path().join("20.git")).unwrap();
  let marker = format!("refs/gity/merge-operations/{}", input.operation_key);
  repo.find_reference(&marker).unwrap().delete().unwrap();
  repo
    .reference(
      "refs/heads/main",
      git2::Oid::from_str(&input.target_sha).unwrap(),
      true,
      "simulate prepare-only crash",
    )
    .unwrap();
  assert!(git
    .merge_operation_outcome("20.git", input.clone())
    .await
    .unwrap()
    .is_none());
  assert_eq!(
    git.merge_operation("20.git", input.clone()).await.unwrap(),
    applied
  );
  repo.find_reference(&marker).unwrap().delete().unwrap();
  repo
    .reference(
      "refs/heads/main",
      git2::Oid::from_str(&input.target_sha).unwrap(),
      true,
      "simulate unproven rewind",
    )
    .unwrap();
  let aborted = git
    .abort_merge_operation("20.git", input.clone(), "inspected and canceled".into(), 5)
    .await
    .unwrap();
  assert!(matches!(aborted, MergeOperationOutcome::Rejected { .. }));
  assert_eq!(
    git.merge_operation("20.git", input.clone()).await.unwrap(),
    aborted
  );
  assert_eq!(
    git.resolve_revision("20.git", "main", "").await.unwrap(),
    input.target_sha
  );
}
#[tokio::test]
async fn target_applied_before_marker_failure_is_recoverable_and_abort_cannot_reverse_it() {
  let (dir, git, input) = fixture().await;
  let applied = git.merge_operation("20.git", input.clone()).await.unwrap();
  let repo = git2::Repository::open_bare(dir.path().join("20.git")).unwrap();
  repo
    .find_reference(&format!(
      "refs/gity/merge-operations/{}",
      input.operation_key
    ))
    .unwrap()
    .delete()
    .unwrap();
  git
    .create_file_commit("20.git", file("main", "later"))
    .await
    .unwrap();
  assert_eq!(
    git
      .merge_operation_outcome("20.git", input.clone())
      .await
      .unwrap(),
    Some(applied.clone())
  );
  assert_eq!(
    git
      .abort_merge_operation("20.git", input, "reconcile".into(), 5)
      .await
      .unwrap(),
    applied
  );
}

#[tokio::test]
async fn native_bundle_and_replica_preserve_receipts_and_merge_commit_gc_root() {
  let (_dir, git, input) = fixture().await;
  let outcome = git.merge_operation("20.git", input.clone()).await.unwrap();
  // Remove all public refs: the receipt parent must retain the exact result.
  git.delete_branch("20.git", "feature").await.unwrap();
  git.delete_branch("20.git", "main").await.unwrap();
  git
    .maintenance("20.git", gity_git::GitMaintenance::Repack)
    .await
    .unwrap();
  let snapshot = git.reference_snapshot("20.git").await.unwrap();
  let bundle = tempfile::NamedTempFile::new().unwrap();
  let mut reader = git
    .spawn_bundle("20.git")
    .await
    .unwrap()
    .into_reader()
    .unwrap();
  let mut output = tokio::fs::File::create(bundle.path()).await.unwrap();
  tokio::io::copy(&mut reader, &mut output).await.unwrap();
  drop(output);
  let target = tempfile::tempdir().unwrap();
  let replica = GitService::new(target.path(), "git");
  replica.init_bare("20.git", "main").await.unwrap();
  replica
    .apply_replica_bundle("20.git", Some(bundle.path()), snapshot)
    .await
    .unwrap();
  assert_eq!(
    replica
      .merge_operation_outcome("20.git", input)
      .await
      .unwrap(),
    Some(outcome.clone())
  );
  let repository = git2::Repository::open_bare(target.path().join("20.git")).unwrap();
  assert!(repository
    .find_commit(git2::Oid::from_str(sha(&outcome)).unwrap())
    .is_ok());
  // Internal retention does not make the object a public revision.
  assert!(replica
    .resolve_revision("20.git", sha(&outcome), "")
    .await
    .is_err());
}

#[path = "merge_operations/public_revisions.rs"]
mod public_revisions;
