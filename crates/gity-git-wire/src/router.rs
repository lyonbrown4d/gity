use crate::NodeClient;

#[async_trait::async_trait]
pub trait GitNodeRouter: Send + Sync {
  async fn begin_write(&self, repository: &str) -> anyhow::Result<NodeClient> {
    self.node_for(repository, true).await
  }
  /// Uncertain operations retain their durable pending marker until fenced repair.
  async fn write_finished(
    &self,
    _repository: &str,
    _node: &NodeClient,
    _confirmed: bool,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  /// A writer must select the fenced primary; readers select current replicas.
  async fn node_for(&self, repository: &str, write: bool) -> anyhow::Result<NodeClient>;
}

pub struct FixedNodeRouter(pub NodeClient);
#[async_trait::async_trait]
impl GitNodeRouter for FixedNodeRouter {
  async fn node_for(&self, _repository: &str, _write: bool) -> anyhow::Result<NodeClient> {
    Ok(self.0.clone())
  }
}
