use crate::{GitCluster, Replica};
use anyhow::{Context, Result, ensure};
use gity_git_wire::{GitOperation, ReferenceSnapshot};

impl GitCluster {
  /// Requires the old primary to be reachable. No automatic election or
  /// data-loss failover is advertised by this primary-ack deployment mode.
  pub async fn promote(&self, project: i64, target: &str) -> Result<crate::Placement> {
    ensure!(
      self.nodes.contains_key(target),
      "target Git node is not configured"
    );
    let repository = format!("{project}.git");
    let mut row = self.placement(&repository).await?;
    if row.state == "ready" {
      let old_node = self.node(&row.primary_node, row.epoch)?;
      let target_node = self.node(target, row.epoch)?;
      let old_health =
        tokio::time::timeout(std::time::Duration::from_secs(5), old_node.health()).await??;
      let target_health =
        tokio::time::timeout(std::time::Duration::from_secs(5), target_node.health()).await??;
      ensure!(
        old_health.node_id == old_node.id()
          && target_health.node_id == target_node.id()
          && target_health.writable,
        "controlled promotion requires reachable matching peers and a writable target"
      );
      let mut moving = row.clone();
      moving.state = "moving".into();
      moving.transition_target = Some(target.into());
      ensure!(
        self.catalog.replace(&row, &moving).await?,
        "placement changed; retry controlled promotion"
      );
      moving.version += 1;
      row = moving;
    }
    ensure!(
      row.state == "moving" && row.transition_target.as_deref() == Some(target),
      "another controlled transition must be recovered first"
    );
    let epoch = row
      .epoch
      .checked_add(1)
      .context("Git placement epoch exhausted")?;
    let old = self.node(&row.primary_node, row.epoch)?;
    let frozen: ReferenceSnapshot = serde_json::from_value(
      old
        .execute(
          &repository,
          GitOperation::FenceRepository {
            next_epoch: epoch as u64,
          },
        )
        .await?,
    )?;
    let target_node = self.node(target, epoch)?;
    if target != row.primary_node {
      let current = target_node
        .execute(&repository, GitOperation::ReferenceSnapshot)
        .await
        .ok()
        .and_then(|value| serde_json::from_value::<ReferenceSnapshot>(value).ok());
      if current
        .as_ref()
        .is_none_or(|current| current.fingerprint() != frozen.fingerprint())
      {
        let copied: ReferenceSnapshot = serde_json::from_value(
          target_node
            .execute(
              &repository,
              GitOperation::SyncReplica {
                source: old.replication_endpoint(),
                source_epoch: epoch as u64,
              },
            )
            .await?,
        )?;
        ensure!(
          copied.fingerprint() == frozen.fingerprint(),
          "final synchronized replica differs from fenced primary"
        );
      }
    }
    let active: ReferenceSnapshot = serde_json::from_value(
      target_node
        .execute(
          &repository,
          GitOperation::ActivatePrimary {
            epoch: epoch as u64,
          },
        )
        .await?,
    )?;
    ensure!(
      active.fingerprint() == frozen.fingerprint(),
      "promoted replica differs from fenced primary"
    );
    let mut complete = row.clone();
    complete.primary_node = target.into();
    complete.transition_target = None;
    complete.epoch = epoch;
    complete.generation += 1;
    complete.pending_writes = 0;
    complete.state = "ready".into();
    complete.last_error = None;
    let mut candidates = std::iter::once(row.primary_node.clone())
      .chain(row.replicas.iter().map(|replica| replica.node.clone()))
      .filter(|node| node != target)
      .collect::<Vec<_>>();
    candidates.dedup();
    complete.replicas = candidates
      .into_iter()
      .take(self.config.replicas.saturating_sub(1))
      .map(Replica::pending)
      .collect();
    ensure!(
      self.catalog.replace(&row, &complete).await?,
      "promotion is fenced but metadata confirmation needs recovery"
    );
    complete.version += 1;
    Ok(complete)
  }
}
