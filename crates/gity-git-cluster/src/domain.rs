use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplicaStatus {
  Pending,
  Syncing,
  Ready,
  Unhealthy,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Replica {
  pub node: String,
  pub generation: i64,
  pub status: ReplicaStatus,
  pub fingerprint: Option<String>,
  pub sync_id: Option<String>,
  pub sync_started_millis: i64,
}
impl Replica {
  pub fn pending(node: String) -> Self {
    Self {
      node,
      generation: -1,
      status: ReplicaStatus::Pending,
      fingerprint: None,
      sync_id: None,
      sync_started_millis: 0,
    }
  }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Placement {
  pub project_id: i64,
  pub primary_node: String,
  pub transition_target: Option<String>,
  pub epoch: i64,
  pub generation: i64,
  pub pending_writes: i64,
  pub state: String,
  pub replicas: Vec<Replica>,
  pub version: i64,
  pub last_error: Option<String>,
}
impl Placement {
  pub fn readable_nodes(&self) -> Vec<&str> {
    let mut nodes = vec![self.primary_node.as_str()];
    if self.state == "ready" && self.pending_writes == 0 {
      nodes.extend(
        self
          .replicas
          .iter()
          .filter(|replica| {
            replica.status == ReplicaStatus::Ready && replica.generation == self.generation
          })
          .map(|replica| replica.node.as_str()),
      );
    }
    nodes
  }
}
pub(crate) fn ranked_nodes(project: i64, nodes: impl Iterator<Item = String>) -> Vec<String> {
  use sha2::{Digest, Sha256};
  let mut ranked = nodes
    .map(|node| {
      let mut hash = Sha256::new();
      hash.update(project.to_le_bytes());
      hash.update(node.as_bytes());
      (hash.finalize().to_vec(), node.clone())
    })
    .collect::<Vec<_>>();
  ranked.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
  ranked.into_iter().map(|(_, node)| node).collect()
}
