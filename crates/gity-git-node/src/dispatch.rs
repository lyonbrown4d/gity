use crate::{GitNodeService, state::RepositoryRole};
use anyhow::{Result, ensure};
use gity_git::{GitMaintenance, GitRpc};
use gity_git_wire::{GitOperation, Maintenance, TransferKind};
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

impl GitNodeService {
  pub(crate) async fn execute_checked(
    &self,
    repo: &str,
    operation: GitOperation,
    request_epoch: u64,
    stop: CancellationToken,
  ) -> Result<Value> {
    match operation {
      GitOperation::InitBare { initial_branch } => {
        self
          .initialize_primary(repo, request_epoch, &initial_branch)
          .await
      }
      GitOperation::ReferenceSnapshot => {
        self.require_repository_state(repo, request_epoch, true)?;
        let snapshot = self.git.reference_snapshot(repo).await?;
        Ok(serde_json::to_value(snapshot)?)
      }
      GitOperation::FenceRepository { next_epoch } => {
        let snapshot = self
          .fence_repository(repo, request_epoch, next_epoch)
          .await?;
        Ok(serde_json::to_value(snapshot)?)
      }
      GitOperation::ActivatePrimary { epoch } => {
        let snapshot = self.activate_primary(repo, request_epoch, epoch).await?;
        Ok(serde_json::to_value(snapshot)?)
      }
      GitOperation::SyncReplica {
        source,
        source_epoch,
      } => {
        let snapshot = self
          .sync_replica(repo, request_epoch, source, source_epoch, stop)
          .await?;
        Ok(serde_json::to_value(snapshot)?)
      }
      operation => {
        let state = self.require_repository_state(repo, request_epoch, false)?;
        if operation.is_write() {
          self.allow_write().map_err(anyhow::Error::msg)?;
          ensure!(
            state.role == RepositoryRole::Primary,
            "repository is not primary"
          );
        }
        self.execute_operation(repo, operation).await
      }
    }
  }

  pub(crate) async fn execute_operation(
    &self,
    repo: &str,
    operation: GitOperation,
  ) -> Result<Value> {
    let git = &self.git;
    match operation {
      GitOperation::InitBare { .. }
      | GitOperation::ReferenceSnapshot
      | GitOperation::SyncReplica { .. }
      | GitOperation::FenceRepository { .. }
      | GitOperation::ActivatePrimary { .. } => {
        anyhow::bail!("control operation must use the state-checked path")
      }
      GitOperation::FileExists {
        revision,
        default,
        path,
      } => Ok(json!(
        git.file_exists(repo, &revision, &default, &path).await?
      )),
      GitOperation::ListBranches { default } => git.list_branches(repo, &default).await,
      GitOperation::ListTags => git.list_tags(repo).await,
      GitOperation::ListTree {
        revision,
        default,
        path,
      } => git.list_tree(repo, &revision, &default, &path).await,
      GitOperation::GetBlob {
        revision,
        default,
        path,
        max_bytes,
      } => {
        let max_bytes = max_bytes.min(4 * 1024 * 1024);
        git
          .get_blob_limited(repo, &revision, &default, &path, max_bytes)
          .await
      }
      GitOperation::GetReadme { revision, default } => {
        git.get_readme(repo, &revision, &default).await
      }
      GitOperation::ListCommits {
        revision,
        default,
        limit,
      } => git.list_commits(repo, &revision, &default, limit).await,
      GitOperation::ResolveRevision { revision, default } => Ok(json!(
        git.resolve_revision(repo, &revision, &default).await?
      )),
      GitOperation::Search {
        revision,
        default,
        parameters,
      } => {
        git
          .search(
            repo,
            &revision,
            &default,
            serde_json::from_value(parameters)?,
          )
          .await
      }
      GitOperation::AnalyzeLanguages { revision, default } => {
        git.analyze_languages(repo, &revision, &default).await
      }
      GitOperation::CreateBranch { name, source } => {
        git.create_branch(repo, &name, &source).await?;
        Ok(Value::Null)
      }
      GitOperation::DeleteBranch { name } => {
        git.delete_branch(repo, &name).await?;
        Ok(Value::Null)
      }
      GitOperation::CreateTag { name, source } => {
        git.create_tag(repo, &name, &source).await?;
        Ok(Value::Null)
      }
      GitOperation::DeleteTag { name } => {
        git.delete_tag(repo, &name).await?;
        Ok(Value::Null)
      }
      GitOperation::CreateFileCommit { input } => Ok(json!(
        git
          .create_file_commit(repo, serde_json::from_value(input)?)
          .await?
      )),
      GitOperation::MergeOperation { input } => Ok(serde_json::to_value(
        git
          .merge_operation(repo, serde_json::from_value(input)?)
          .await?,
      )?),
      GitOperation::MergeOperationOutcome { input } => Ok(serde_json::to_value(
        git
          .merge_operation_outcome(repo, serde_json::from_value(input)?)
          .await?,
      )?),
      GitOperation::AbortMergeOperation {
        input,
        reason,
        actor_user_id,
      } => Ok(serde_json::to_value(
        git
          .abort_merge_operation(repo, serde_json::from_value(input)?, reason, actor_user_id)
          .await?,
      )?),
      GitOperation::MergeBranches { input } => {
        git
          .merge_branches(repo, serde_json::from_value(input)?)
          .await?;
        Ok(Value::Null)
      }
      GitOperation::DiffBranches { target, source } => {
        Ok(json!(git.diff_branches(repo, &target, &source).await?))
      }
      GitOperation::Maintenance { kind } => {
        git
          .maintenance(
            repo,
            match kind {
              Maintenance::GarbageCollect => GitMaintenance::GarbageCollect,
              Maintenance::Repack => GitMaintenance::Repack,
              Maintenance::UpdateServerInfo => GitMaintenance::UpdateServerInfo,
            },
          )
          .await?;
        Ok(Value::Null)
      }
      GitOperation::InstallManagedHook => {
        git
          .ensure_update_hook(repo, &self.settings.hook_executable)
          .await?;
        Ok(Value::Null)
      }
    }
  }
  pub(crate) async fn open_transfer(
    &self,
    repo: &str,
    kind: TransferKind,
  ) -> Result<gity_git::GitProcess> {
    match kind {
      TransferKind::Http {
        receive,
        advertise,
        environment,
      } => {
        self
          .git
          .spawn_rpc(
            repo,
            if receive {
              GitRpc::ReceivePack
            } else {
              GitRpc::UploadPack
            },
            advertise,
            environment,
          )
          .await
      }
      TransferKind::Ssh {
        receive,
        protocol,
        policy,
      } => {
        let policy = policy.map(serde_json::from_value).transpose()?;
        self
          .git
          .spawn_ssh(
            repo,
            if receive {
              GitRpc::ReceivePack
            } else {
              GitRpc::UploadPack
            },
            protocol.as_deref(),
            policy.as_ref(),
          )
          .await
      }
      TransferKind::Archive { revision } => self.git.spawn_archive(repo, &revision).await,
      TransferKind::Bundle => anyhow::bail!("bundle transfers use the replication path"),
    }
  }
}
