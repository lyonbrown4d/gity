use crate::{Placement, domain::ranked_nodes, repository::Catalog};
use anyhow::{Context, Result, bail, ensure};
use gity_core::GitClusterConfig;
use gity_git_wire::{GitNodeRouter, NodeClient, NodeEndpoint, RepositoryKey};
use sea_orm::DatabaseConnection;
use std::{
  collections::BTreeMap,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::Duration,
};

#[derive(Clone)]
pub struct GitCluster {
  pub(crate) catalog: Catalog,
  pub(crate) nodes: Arc<BTreeMap<String, NodeClient>>,
  pub(crate) config: GitClusterConfig,
  reads: Arc<AtomicUsize>,
}
impl GitCluster {
  pub fn new(connection: DatabaseConnection, config: GitClusterConfig) -> Result<Self> {
    config.validate()?;
    let mut nodes = BTreeMap::new();
    for node in &config.nodes {
      let read =
        |path: &Option<std::path::PathBuf>| path.as_ref().map(std::fs::read_to_string).transpose();
      nodes.insert(
        node.id.clone(),
        NodeClient::new(NodeEndpoint {
          id: node.id.clone(),
          uri: node.endpoint.clone(),
          token: node.token.clone(),
          ca_certificate: read(&node.ca_file)?,
          client_certificate: read(&node.client_certificate_file)?,
          client_key: read(&node.client_key_file)?,
        })?,
      );
    }
    Ok(Self {
      catalog: Catalog { connection },
      nodes: Arc::new(nodes),
      config,
      reads: Arc::new(AtomicUsize::new(0)),
    })
  }
  pub async fn placement(&self, repository: &str) -> Result<Placement> {
    let project = project_id(repository)?;
    if let Some(row) = self.catalog.get(project).await? {
      return Ok(row);
    }
    self
      .catalog
      .create(
        project,
        ranked_nodes(project, self.nodes.keys().cloned()),
        self.config.replicas,
      )
      .await
  }
  pub(crate) fn node(&self, id: &str, epoch: i64) -> Result<NodeClient> {
    ensure!(epoch > 0, "invalid Git placement epoch");
    Ok(
      self
        .nodes
        .get(id)
        .with_context(|| {
          format!("Git storage node {id} is not configured; placements are never silently remapped")
        })?
        .clone()
        .with_epoch(epoch as u64),
    )
  }
  pub async fn inspect(&self, project: i64) -> Result<Placement> {
    self.placement(&format!("{project}.git")).await
  }
}
#[async_trait::async_trait]
impl GitNodeRouter for GitCluster {
  async fn node_for(&self, repository: &str, write: bool) -> Result<NodeClient> {
    let row = self.placement(repository).await?;
    ensure!(
      row.state == "ready",
      "repository is undergoing controlled placement recovery"
    );
    if write {
      return self.node(&row.primary_node, row.epoch);
    }
    let candidates = row.readable_nodes();
    let start = self.reads.fetch_add(1, Ordering::Relaxed) % candidates.len();
    for offset in 0..candidates.len() {
      let node = self.node(candidates[(start + offset) % candidates.len()], row.epoch)?;
      if let Ok(Ok(health)) = tokio::time::timeout(Duration::from_secs(2), node.health()).await
        && health.node_id == node.id()
      {
        return Ok(node);
      }
    }
    bail!("no current healthy Git replica is available")
  }
  async fn begin_write(&self, repository: &str) -> Result<NodeClient> {
    for _ in 0..16 {
      let row = self.placement(repository).await?;
      ensure!(
        row.state == "ready",
        "repository is undergoing controlled placement recovery"
      );
      let node = self.node(&row.primary_node, row.epoch)?;
      if self.catalog.begin_write(&row).await? {
        return Ok(node);
      }
      tokio::task::yield_now().await;
    }
    bail!("Git placement changed too often; retry the operation")
  }
  async fn write_finished(
    &self,
    repository: &str,
    node: &NodeClient,
    confirmed: bool,
  ) -> Result<()> {
    if confirmed {
      self
        .catalog
        .finish_write(project_id(repository)?, node.epoch() as i64, node.id())
        .await?;
    } else {
      tracing::warn!(
        repository,
        node = node.id(),
        "Git outcome is uncertain; pending marker retains primary-only reads until fenced reconciliation"
      );
    }
    Ok(())
  }
}
fn project_id(repository: &str) -> Result<i64> {
  let key = RepositoryKey::try_from(repository.to_owned())?;
  Ok(
    key
      .as_str()
      .strip_suffix(".git")
      .context("invalid repository key")?
      .parse()?,
  )
}
