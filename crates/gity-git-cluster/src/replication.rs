use crate::{GitCluster, Placement, ReplicaStatus};
use anyhow::{Context, Result, ensure};
use gity_git_wire::{GitOperation, ReferenceSnapshot};

impl GitCluster {
  /// Primary acknowledgments are independent of asynchronous replica catch-up.
  pub async fn replicate_tick(&self) -> Result<usize> {
    let mut completed = 0;
    for row in self.catalog.scan(100).await? {
      if row.state != "ready" || row.pending_writes != 0 {
        continue;
      }
      for replica in &row.replicas {
        if replica.status == ReplicaStatus::Ready && replica.generation == row.generation {
          continue;
        }
        if replica.status == ReplicaStatus::Syncing
          && replica.sync_started_millis > chrono::Utc::now().timestamp_millis() - 3_600_000
        {
          continue;
        }
        let sync_id = gity_ids::new_operation_id().to_string();
        let Some(claim) = self.claim_sync(&row, &replica.node, &sync_id).await? else {
          continue;
        };
        let result = self.copy_replica(&claim, &replica.node).await;
        self
          .finish_sync(&claim, &replica.node, &sync_id, result)
          .await?;
        completed += 1;
        // A fresh catalog snapshot is required before another replica is claimed.
        break;
      }
    }
    Ok(completed)
  }
  async fn claim_sync(
    &self,
    row: &Placement,
    node: &str,
    sync_id: &str,
  ) -> Result<Option<Placement>> {
    let mut claimed = row.clone();
    let replica = claimed
      .replicas
      .iter_mut()
      .find(|replica| replica.node == node)
      .context("replica assignment disappeared")?;
    replica.status = ReplicaStatus::Syncing;
    replica.sync_id = Some(sync_id.to_owned());
    replica.sync_started_millis = chrono::Utc::now().timestamp_millis();
    if self.catalog.replace(row, &claimed).await? {
      claimed.version += 1;
      Ok(Some(claimed))
    } else {
      Ok(None)
    }
  }
  async fn copy_replica(&self, row: &Placement, target: &str) -> Result<String> {
    let repository = format!("{}.git", row.project_id);
    let source = self.node(&row.primary_node, row.epoch)?;
    let before: ReferenceSnapshot = serde_json::from_value(
      source
        .execute(&repository, GitOperation::ReferenceSnapshot)
        .await?,
    )?;
    let destination = self.node(target, row.epoch)?;
    let copied: ReferenceSnapshot = serde_json::from_value(
      destination
        .execute(
          &repository,
          GitOperation::SyncReplica {
            source: source.replication_endpoint(),
            source_epoch: source.epoch(),
          },
        )
        .await?,
    )?;
    ensure!(
      copied.fingerprint() == before.fingerprint(),
      "primary changed while the replica was being synchronized"
    );
    Ok(copied.fingerprint())
  }
  async fn finish_sync(
    &self,
    claim: &Placement,
    node: &str,
    sync_id: &str,
    result: Result<String>,
  ) -> Result<()> {
    for _ in 0..16 {
      let Some(current) = self.catalog.get(claim.project_id).await? else {
        return Ok(());
      };
      if current.epoch != claim.epoch {
        return Ok(());
      }
      let mut updated = current.clone();
      let Some(replica) = updated
        .replicas
        .iter_mut()
        .find(|replica| replica.node == node && replica.sync_id.as_deref() == Some(sync_id))
      else {
        return Ok(());
      };
      let ready = result.is_ok()
        && current.state == "ready"
        && current.pending_writes == 0
        && current.generation == claim.generation;
      replica.status = if ready {
        ReplicaStatus::Ready
      } else {
        ReplicaStatus::Pending
      };
      replica.generation = if ready { claim.generation } else { -1 };
      replica.fingerprint = result.as_ref().ok().cloned();
      replica.sync_id = None;
      updated.last_error = result.as_ref().err().map(|error| error.to_string());
      if self.catalog.replace(&current, &updated).await? {
        return Ok(());
      }
    }
    anyhow::bail!("replica metadata changed too often; synchronization result was not acknowledged")
  }
}
