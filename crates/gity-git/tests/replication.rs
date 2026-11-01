use gity_git::GitService;
use std::path::Path;
use tempfile::TempDir;
use tokio::io::{copy, AsyncWriteExt};

async fn services() -> (TempDir, TempDir, GitService, GitService) {
  let source_root = tempfile::tempdir().unwrap();
  let target_root = tempfile::tempdir().unwrap();
  let source = GitService::new(source_root.path(), "git");
  let target = GitService::new(target_root.path(), "git");
  source.init_bare("source.git", "main").await.unwrap();
  target.init_bare("target.git", "main").await.unwrap();
  (source_root, target_root, source, target)
}

async fn write_bundle(service: &GitService, repo: &str, path: &Path) {
  let mut reader = service
    .spawn_bundle(repo)
    .await
    .unwrap()
    .into_reader()
    .unwrap();
  let mut file = tokio::fs::File::create(path).await.unwrap();
  copy(&mut reader, &mut file).await.unwrap();
  file.flush().await.unwrap();
}

async fn commit(service: &GitService, name: &str, content: &str) {
  service
    .create_file_commit(
      "source.git",
      gity_git::CreateFileCommitInput {
        branch_name: "main".into(),
        file_path: name.into(),
        content: content.into(),
        message: name.into(),
        author_name: "Replica Test".into(),
        author_email: "replica@example.org".into(),
      },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn initial_sync_and_later_ref_deletions_match_source() {
  let (_source_root, _target_root, source, target) = services().await;
  commit(&source, "one.txt", "one").await;
  source.create_tag("source.git", "v1", "main").await.unwrap();
  source
    .create_branch("source.git", "release", "main")
    .await
    .unwrap();
  let bundle = tempfile::NamedTempFile::new().unwrap();
  let snapshot = source.reference_snapshot("source.git").await.unwrap();
  write_bundle(&source, "source.git", bundle.path()).await;
  target
    .apply_replica_bundle("target.git", Some(bundle.path()), snapshot.clone())
    .await
    .unwrap();
  assert_eq!(
    target.reference_snapshot("target.git").await.unwrap(),
    snapshot
  );

  commit(&source, "two.txt", "two").await;
  source.delete_tag("source.git", "v1").await.unwrap();
  source.delete_branch("source.git", "release").await.unwrap();
  let updated = source.reference_snapshot("source.git").await.unwrap();
  write_bundle(&source, "source.git", bundle.path()).await;
  target
    .apply_replica_bundle("target.git", Some(bundle.path()), updated.clone())
    .await
    .unwrap();
  assert_eq!(
    target.reference_snapshot("target.git").await.unwrap(),
    updated
  );
}

#[tokio::test]
async fn empty_snapshot_clears_refs_and_preserves_symbolic_head() {
  let (_source_root, _target_root, source, target) = services().await;
  commit(&source, "one.txt", "one").await;
  let bundle = tempfile::NamedTempFile::new().unwrap();
  let populated = source.reference_snapshot("source.git").await.unwrap();
  write_bundle(&source, "source.git", bundle.path()).await;
  target
    .apply_replica_bundle("target.git", Some(bundle.path()), populated)
    .await
    .unwrap();

  source.delete_branch("source.git", "main").await.unwrap();
  let empty = source.reference_snapshot("source.git").await.unwrap();
  assert!(empty.refs.is_empty());
  target
    .apply_replica_bundle("target.git", None, empty.clone())
    .await
    .unwrap();
  assert_eq!(
    target.reference_snapshot("target.git").await.unwrap(),
    empty
  );
}

#[tokio::test]
async fn detached_head_is_preserved() {
  let (_source_root, _target_root, source, target) = services().await;
  commit(&source, "one.txt", "one").await;
  let source_repo =
    git2::Repository::open_bare(source.repository_path("source.git").unwrap()).unwrap();
  let target_oid = source_repo
    .find_reference("refs/heads/main")
    .unwrap()
    .target()
    .unwrap();
  source_repo
    .reference("HEAD", target_oid, true, "detach replication test")
    .unwrap();

  let bundle = tempfile::NamedTempFile::new().unwrap();
  let snapshot = source.reference_snapshot("source.git").await.unwrap();
  assert!(matches!(
    &snapshot.head,
    gity_git_wire::RepositoryHead::Detached(_)
  ));
  write_bundle(&source, "source.git", bundle.path()).await;
  target
    .apply_replica_bundle("target.git", Some(bundle.path()), snapshot.clone())
    .await
    .unwrap();
  assert_eq!(
    target.reference_snapshot("target.git").await.unwrap(),
    snapshot
  );
}

#[tokio::test]
async fn damaged_bundle_does_not_change_visible_refs() {
  let (_source_root, _target_root, source, target) = services().await;
  commit(&source, "one.txt", "one").await;
  let bundle = tempfile::NamedTempFile::new().unwrap();
  let initial = source.reference_snapshot("source.git").await.unwrap();
  write_bundle(&source, "source.git", bundle.path()).await;
  target
    .apply_replica_bundle("target.git", Some(bundle.path()), initial)
    .await
    .unwrap();
  let before = target.reference_snapshot("target.git").await.unwrap();

  commit(&source, "two.txt", "two").await;
  let updated = source.reference_snapshot("source.git").await.unwrap();
  write_bundle(&source, "source.git", bundle.path()).await;
  tokio::fs::write(bundle.path(), b"not a git bundle")
    .await
    .unwrap();
  assert!(target
    .apply_replica_bundle("target.git", Some(bundle.path()), updated)
    .await
    .is_err());
  assert_eq!(
    target.reference_snapshot("target.git").await.unwrap(),
    before
  );
}
