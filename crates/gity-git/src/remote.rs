use crate::GitService;
use anyhow::Result;
use gity_git_wire::{GitNodeRouter, GitOperation};
use serde::de::DeserializeOwned;
use std::sync::Arc;

impl GitService {
  pub fn with_remote(router: Arc<dyn GitNodeRouter>) -> Self {
    let mut service = Self::new(std::path::PathBuf::new(), std::path::PathBuf::new());
    service.remote = Some(router);
    service
  }
  pub fn is_remote(&self) -> bool {
    self.remote.is_some()
  }

  pub(crate) async fn execute_remote<T: DeserializeOwned>(
    &self,
    repo: &str,
    operation: GitOperation,
  ) -> Result<T> {
    let router = self.remote.as_ref().expect("remote backend selected");
    let write = operation.is_write();
    let node = if write {
      router.begin_write(repo).await?
    } else {
      router.node_for(repo, false).await?
    };
    let result = node.execute(repo, operation).await;
    if write {
      let confirmed = result
        .as_ref()
        .err()
        .is_none_or(gity_git_wire::definitive_rejection);
      router.write_finished(repo, &node, confirmed).await?;
    }
    Ok(serde_json::from_value(result?)?)
  }
}
