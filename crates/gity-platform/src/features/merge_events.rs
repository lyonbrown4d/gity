use super::*;
use gity_entities::merge_operations;
use gity_git::{MergeOperationInput, MergeOperationOutcome};

pub(super) fn input(op: &merge_operations::Model) -> MergeOperationInput {
  MergeOperationInput {
    operation_key: op.operation_key.clone(),
    actor_user_id: op.actor_user_id,
    source_branch: op.source_branch.clone(),
    target_branch: op.target_branch.clone(),
    source_sha: op.source_sha.clone(),
    target_sha: op.target_sha.clone(),
    message: op.message.clone(),
    author_name: op.author_name.clone(),
    author_email: op.author_email.clone(),
    timestamp: op.created_at.and_utc().timestamp(),
  }
}
#[cfg(test)]
pub(super) async fn finish(
  s: &AppState,
  id: i64,
  outcome: MergeOperationOutcome,
) -> Result<merge_operations::Model, AppError> {
  settle(s.collaboration.lock_merge_operation(id).await?, outcome).await
}

pub(super) async fn settle(
  guard: gity_collaboration::MergeOperationGuard,
  outcome: MergeOperationOutcome,
) -> Result<merge_operations::Model, AppError> {
  guard
    .finish(match outcome {
      MergeOperationOutcome::Applied { commit_sha } => Ok(commit_sha),
      MergeOperationOutcome::Rejected {
        reason,
        actor_user_id,
      } => Err(gity_collaboration::MergeRejection {
        reason,
        actor_user_id,
      }),
    })
    .await
}

pub(super) async fn reconcile(
  s: &AppState,
  id: i64,
) -> Result<Option<merge_operations::Model>, AppError> {
  let op = s.collaboration.merge_operation(id).await?;
  let repo = format!("{}.git", op.project_id);
  let transport = s
    .git
    .prepare_merge_transport(&repo, false)
    .await
    .map_err(|error| AppError::conflict(error.to_string()))?;
  let guard = s.collaboration.lock_merge_operation(id).await?;
  let op = guard.operation();
  if op.status != "pending" {
    return Ok(Some(guard.release().await?));
  }
  let outcome = transport
    .git()
    .merge_operation_outcome(&repo, input(op))
    .await
    .map_err(|error| AppError::conflict(error.to_string()))?;
  match outcome {
    Some(outcome) => Ok(Some(settle(guard, outcome).await?)),
    None => {
      guard.release().await?;
      Ok(None)
    }
  }
}

pub(super) async fn apply(s: &AppState, id: i64) -> Result<merge_operations::Model, AppError> {
  execute(s, id, None).await
}

pub(super) async fn abort_operation(
  s: &AppState,
  id: i64,
  reason: String,
  actor_user_id: i64,
) -> Result<merge_operations::Model, AppError> {
  execute(s, id, Some((reason, actor_user_id))).await
}

async fn execute(
  s: &AppState,
  id: i64,
  abort: Option<(String, i64)>,
) -> Result<merge_operations::Model, AppError> {
  let op = s.collaboration.merge_operation(id).await?;
  let repo = format!("{}.git", op.project_id);
  let transport = s
    .git
    .prepare_merge_transport(&repo, true)
    .await
    .map_err(|error| AppError::conflict(error.to_string()))?;
  let mut confirmed = true;
  let result = async {
    let guard = s.collaboration.lock_merge_operation(id).await?;
    let op = guard.operation();
    if op.status != "pending" {
      return guard.release().await;
    }
    let outcome = match abort {
      Some((reason, actor)) => {
        transport
          .git()
          .abort_merge_operation(&repo, input(op), reason, actor)
          .await
      }
      None => transport.git().merge_operation(&repo, input(op)).await,
    };
    confirmed = transport.outcome_is_confirmed(&outcome);
    let outcome = outcome
      .map_err(|error| AppError::conflict(format!("merge operation remains pending: {error}")))?;
    settle(guard, outcome).await
  }
  .await;
  transport
    .finish(confirmed)
    .await
    .map_err(|error| AppError::conflict(error.to_string()))?;
  result
}

pub(super) async fn deliver(s: &AppState, op: &merge_operations::Model) -> Result<(), AppError> {
  if op.status != "applied" || op.events_delivered {
    return Ok(());
  }
  let sha = op
    .result_sha
    .as_deref()
    .ok_or_else(|| AppError::conflict("merge result receipt missing"))?;
  // Audit and notification were committed with the MR. Push pipelines have a
  // database uniqueness key, so at-least-once outbox delivery is safe.
  crate::ci::create_push_pipeline(s, op.project_id, &op.target_branch, sha).await?;
  s.collaboration.merge_events_delivered(op.id).await
}

/// Maintenance reads existing Git proof only. It never retries authorized writes
/// on behalf of an actor who may since have lost access.
pub(crate) async fn recover(s: &AppState) -> Result<(), AppError> {
  let mut cursor = 0;
  loop {
    let rows = s
      .collaboration
      .pending_merge_operations(cursor, 100)
      .await?;
    if rows.is_empty() {
      break;
    }
    for op in rows {
      cursor = op.id;
      let result = async {
        let op = if op.status == "pending" {
          let Some(completed) = reconcile(s, op.id).await? else {
            return Ok::<_, AppError>(());
          };
          completed
        } else {
          op
        };
        deliver(s, &op).await
      }
      .await;
      if let Err(error) = result {
        tracing::warn!(operation_id=cursor, error=%error, "merge recovery remains pending");
      }
    }
  }
  Ok(())
}

#[cfg(test)]
#[path = "merge_recovery_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "merge_guard_tests.rs"]
mod guard_tests;
