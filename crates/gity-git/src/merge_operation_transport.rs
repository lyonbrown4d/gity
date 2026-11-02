use crate::GitService;
use anyhow::Result;
use gity_git_wire::{FixedNodeRouter, GitNodeRouter, NodeClient};
use std::sync::Arc;

/// Placement accounting runs outside an application's merge journal transaction.
/// The selected service performs only local Git or fixed-node RPC while locked.
pub struct MergeOperationTransport {
  service: GitService,
  completion: Option<(Arc<dyn GitNodeRouter>, NodeClient, String)>,
}

impl GitService {
  pub async fn prepare_merge_transport(
    &self,
    repo: &str,
    write: bool,
  ) -> Result<MergeOperationTransport> {
    let Some(router) = &self.remote else {
      return Ok(MergeOperationTransport {
        service: self.clone(),
        completion: None,
      });
    };
    let node = if write {
      router.begin_write(repo).await?
    } else {
      router.node_for(repo, false).await?
    };
    Ok(MergeOperationTransport {
      service: GitService::with_remote(Arc::new(FixedNodeRouter(node.clone()))),
      completion: write.then(|| (router.clone(), node, repo.to_owned())),
    })
  }
}

impl MergeOperationTransport {
  pub fn outcome_is_confirmed<T>(&self, result: &Result<T>) -> bool {
    result
      .as_ref()
      .err()
      .is_none_or(gity_git_wire::definitive_rejection)
  }

  pub fn git(&self) -> &GitService {
    &self.service
  }

  /// Call after releasing the journal transaction. Uncertain RPC outcomes keep
  /// the cluster's existing durable pending-write marker for fenced recovery.
  pub async fn finish(self, confirmed: bool) -> Result<()> {
    if let Some((router, node, repo)) = self.completion {
      router.write_finished(&repo, &node, confirmed).await?;
    }
    Ok(())
  }
}
