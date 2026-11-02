use super::*;
use gity_git::GitService;
use gity_git_wire::{GitNodeRouter, NodeClient};
use std::sync::{
  Arc,
  atomic::{AtomicUsize, Ordering},
};

struct AccountingRouter {
  node: NodeClient,
  begins: AtomicUsize,
  reads: AtomicUsize,
  finishes: AtomicUsize,
}
#[tonic::async_trait]
impl GitNodeRouter for AccountingRouter {
  async fn node_for(&self, _: &str, _: bool) -> anyhow::Result<NodeClient> {
    self.reads.fetch_add(1, Ordering::SeqCst);
    Ok(self.node.clone())
  }
  async fn begin_write(&self, _: &str) -> anyhow::Result<NodeClient> {
    self.begins.fetch_add(1, Ordering::SeqCst);
    Ok(self.node.clone())
  }
  async fn write_finished(&self, _: &str, _: &NodeClient, confirmed: bool) -> anyhow::Result<()> {
    assert!(confirmed);
    self.finishes.fetch_add(1, Ordering::SeqCst);
    Ok(())
  }
}

#[tokio::test]
async fn prepared_merge_transport_keeps_catalog_accounting_outside_fixed_node_rpc() {
  let root = tempfile::tempdir().unwrap();
  let node = support::start_node(support::settings(
    "merge-transport",
    root.path().join("repos"),
    "git".into(),
  ))
  .await;
  let direct = support::remote(node.client.clone());
  direct.init_bare("42.git", "main").await.unwrap();
  let target = direct
    .create_file_commit("42.git", file("main", "base"))
    .await
    .unwrap();
  direct
    .create_branch("42.git", "feature", "main")
    .await
    .unwrap();
  let source = direct
    .create_file_commit("42.git", file("feature", "feature"))
    .await
    .unwrap();
  let input = MergeOperationInput {
    operation_key: "019a3955-1000-7000-8000-000000000011".into(),
    actor_user_id: 1,
    source_branch: "feature".into(),
    target_branch: "main".into(),
    source_sha: source,
    target_sha: target,
    message: "merge".into(),
    author_name: "Tester".into(),
    author_email: "test@example.test".into(),
    timestamp: 1_800_000_000,
  };
  let router = Arc::new(AccountingRouter {
    node: node.client.clone(),
    begins: AtomicUsize::new(0),
    reads: AtomicUsize::new(0),
    finishes: AtomicUsize::new(0),
  });
  let service = GitService::with_remote(router.clone());
  let prepared = service
    .prepare_merge_transport("42.git", true)
    .await
    .unwrap();
  assert_eq!(router.begins.load(Ordering::SeqCst), 1);
  let outcome = prepared
    .git()
    .merge_operation("42.git", input.clone())
    .await;
  assert_eq!(router.begins.load(Ordering::SeqCst), 1);
  assert_eq!(router.finishes.load(Ordering::SeqCst), 0);
  assert!(prepared.outcome_is_confirmed(&outcome));
  let outcome = outcome.unwrap();
  prepared.finish(true).await.unwrap();
  assert_eq!(router.finishes.load(Ordering::SeqCst), 1);
  let prepared = service
    .prepare_merge_transport("42.git", false)
    .await
    .unwrap();
  assert_eq!(router.reads.load(Ordering::SeqCst), 1);
  assert_eq!(
    prepared
      .git()
      .merge_operation_outcome("42.git", input)
      .await
      .unwrap(),
    Some(outcome)
  );
  prepared.finish(true).await.unwrap();
  assert_eq!(router.reads.load(Ordering::SeqCst), 1);
  assert_eq!(router.finishes.load(Ordering::SeqCst), 1);
  node.stop().await;
}
