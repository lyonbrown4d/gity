use crate::{
  merge_operation_model::{MergeOperationInput, MergeOperationOutcome},
  merge_operation_receipt as receipt, GitService,
};
use anyhow::{ensure, Result};
use git2::{Oid, Repository};
use gity_git_wire::GitOperation;

impl GitService {
  pub async fn merge_operation_outcome(
    &self,
    repo: &str,
    input: MergeOperationInput,
  ) -> Result<Option<MergeOperationOutcome>> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          GitOperation::MergeOperationOutcome {
            input: serde_json::to_value(input)?,
          },
        )
        .await;
    }
    let value = self
      .read(repo, move |r| {
        Ok(serde_json::to_value(receipt::observed(&r, &input)?)?)
      })
      .await?;
    Ok(serde_json::from_value(value)?)
  }

  pub async fn merge_operation(
    &self,
    repo: &str,
    input: MergeOperationInput,
  ) -> Result<MergeOperationOutcome> {
    self.run_merge_operation(repo, input, None).await
  }

  /// Explicit, freshly authorized operator action. Aborting first reconciles
  /// proof of application; otherwise its immutable receipt fences stale retries.
  pub async fn abort_merge_operation(
    &self,
    repo: &str,
    input: MergeOperationInput,
    reason: String,
    actor_user_id: i64,
  ) -> Result<MergeOperationOutcome> {
    ensure!(!reason.trim().is_empty(), "abort reason is required");
    self
      .run_merge_operation(repo, input, Some((reason, actor_user_id)))
      .await
  }

  async fn run_merge_operation(
    &self,
    repo: &str,
    input: MergeOperationInput,
    abort: Option<(String, i64)>,
  ) -> Result<MergeOperationOutcome> {
    if self.remote.is_some() {
      let input = serde_json::to_value(input)?;
      return self
        .execute_remote(
          repo,
          match abort {
            Some((reason, actor_user_id)) => GitOperation::AbortMergeOperation {
              input,
              reason,
              actor_user_id,
            },
            None => GitOperation::MergeOperation { input },
          },
        )
        .await;
    }
    let value = self
      .read(repo, move |r| {
        let target = crate::mutations::validate_ref(&input.target_branch, "heads")?;
        let source = crate::mutations::validate_ref(&input.source_branch, "heads")?;
        ensure!(source != target, "merge source and target must differ");
        let marker = input.marker()?;
        // git2 transactions are NOT atomic across refs. Keep the marker lock
        // across a separate, single-target-ref transaction, then publish outcome.
        // All callers lock marker first (refs/gity sorts before refs/heads).
        let mut guard = r.transaction()?;
        guard.lock_ref(&marker)?;
        if let Some(outcome) = receipt::read(&r, &marker, &input)? {
          return Ok(serde_json::to_value(outcome)?);
        }
        let mut branch_tx = r.transaction()?;
        let mut refs = [source.as_str(), target.as_str()];
        refs.sort_unstable();
        for name in refs {
          branch_tx.lock_ref(name)?;
        }
        let outcome = if let Some(outcome) = receipt::observed(&r, &input)? {
          outcome
        } else if let Some((reason, actor_user_id)) = abort {
          MergeOperationOutcome::Rejected {
            reason: format!("operator aborted: {reason}"),
            actor_user_id,
          }
        } else {
          let old = tip(&r, &target)?;
          let head = tip(&r, &source)?;
          let prepared = receipt::read(&r, &input.prepared(), &input)?;
          let pins_match = old
            .as_ref()
            .is_some_and(|commit| commit.id().to_string() == input.target_sha)
            && head
              .as_ref()
              .is_some_and(|commit| commit.id().to_string() == input.source_sha);
          if !pins_match {
            ensure!(
              prepared.is_none(),
              "merge outcome requires operator reconciliation; pinned branches changed"
            );
            MergeOperationOutcome::Rejected {
              reason: "merge source or target changed; refresh required".into(),
              actor_user_id: input.actor_user_id,
            }
          } else {
            let candidate = match prepared {
              Some(MergeOperationOutcome::Applied { commit_sha }) => {
                Ok(Oid::from_str(&commit_sha)?)
              }
              Some(_) => anyhow::bail!("invalid prepared merge receipt"),
              None => candidate(
                &r,
                &input,
                old.as_ref().expect("target pin matched"),
                head.as_ref().expect("source pin matched"),
              ),
            };
            match candidate {
              Ok(candidate) => {
                let outcome = MergeOperationOutcome::Applied {
                  commit_sha: candidate.to_string(),
                };
                if receipt::read(&r, &input.prepared(), &input)?.is_none() {
                  let prepared = receipt::object(&r, &input, &outcome)?;
                  r.reference(
                    &input.prepared(),
                    prepared,
                    false,
                    "Prepare Gity merge operation",
                  )?;
                }
                branch_tx.set_target(&target, candidate, None, "Apply Gity merge operation")?;
                branch_tx.commit()?;
                let object = receipt::object(&r, &input, &outcome)?;
                guard.set_target(&marker, object, None, "Record Gity merge outcome")?;
                guard.commit()?;
                return Ok(serde_json::to_value(outcome)?);
              }
              Err(error) if error.to_string() == "git merge conflict" => {
                MergeOperationOutcome::Rejected {
                  reason: error.to_string(),
                  actor_user_id: input.actor_user_id,
                }
              }
              Err(error) => return Err(error),
            }
          }
        };
        let object = receipt::object(&r, &input, &outcome)?;
        guard.set_target(&marker, object, None, "Record Gity merge outcome")?;
        guard.commit()?;
        Ok(serde_json::to_value(outcome)?)
      })
      .await?;
    Ok(serde_json::from_value(value)?)
  }
}

fn candidate(
  r: &Repository,
  input: &MergeOperationInput,
  target: &git2::Commit<'_>,
  source: &git2::Commit<'_>,
) -> Result<Oid> {
  if target.id() == source.id() || r.graph_descendant_of(target.id(), source.id())? {
    return Ok(target.id());
  }
  let mut index = r.merge_commits(target, source, None)?;
  ensure!(!index.has_conflicts(), "git merge conflict");
  let tree = r.find_tree(index.write_tree_to(r)?)?;
  let signature = git2::Signature::new(
    &input.author_name,
    &input.author_email,
    &git2::Time::new(input.timestamp, 0),
  )?;
  Ok(r.commit(
    None,
    &signature,
    &signature,
    &format!(
      "{}\n\nGity-Merge-Operation: {}",
      input.message, input.operation_key
    ),
    &tree,
    &[target, source],
  )?)
}

fn tip<'a>(repo: &'a Repository, name: &str) -> Result<Option<git2::Commit<'a>>> {
  match repo
    .find_reference(name)
    .and_then(|reference| reference.peel_to_commit())
  {
    Ok(commit) => Ok(Some(commit)),
    Err(error) if error.code() == git2::ErrorCode::NotFound => Ok(None),
    Err(error) => Err(error.into()),
  }
}
