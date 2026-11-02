use super::merge_operation_model::{MergeOperationInput, MergeOperationOutcome, Receipt};
use anyhow::{ensure, Context, Result};
use git2::{Oid, Repository};

pub(super) fn read(
  repo: &Repository,
  reference: &str,
  input: &MergeOperationInput,
) -> Result<Option<MergeOperationOutcome>> {
  let reference = match repo.find_reference(reference) {
    Ok(reference) => reference,
    Err(error) if error.code() == git2::ErrorCode::NotFound => return Ok(None),
    Err(error) => return Err(error.into()),
  };
  let commit = reference.peel_to_commit()?;
  let tree = commit.tree()?;
  let entry = tree
    .get_name("receipt.json")
    .context("merge receipt missing")?;
  let blob = repo.find_blob(entry.id())?;
  let receipt: Receipt = serde_json::from_slice(blob.content())?;
  ensure!(
    &receipt.input == input,
    "merge operation parameters do not match durable receipt"
  );
  Ok(Some(receipt.outcome))
}

pub(super) fn object(
  repo: &Repository,
  input: &MergeOperationInput,
  outcome: &MergeOperationOutcome,
) -> Result<Oid> {
  let bytes = serde_json::to_vec(&Receipt {
    input: input.clone(),
    outcome: outcome.clone(),
  })?;
  let blob = repo.blob(&bytes)?;
  let mut tree = repo.treebuilder(None)?;
  tree.insert("receipt.json", blob, 0o100644)?;
  let tree = repo.find_tree(tree.write()?)?;
  let signature = git2::Signature::new(
    "Gity merge journal",
    "merge@gity.local",
    &git2::Time::new(input.timestamp, 0),
  )?;
  let parent = match outcome {
    MergeOperationOutcome::Applied { commit_sha } => {
      Some(repo.find_commit(Oid::from_str(commit_sha)?)?)
    }
    MergeOperationOutcome::Rejected { .. } => None,
  };
  let parents = parent.iter().collect::<Vec<_>>();
  // A commit parent, rather than a JSON-only SHA, makes the exact merge result
  // reachable to native bundle --all, replication and garbage collection.
  Ok(repo.commit(
    None,
    &signature,
    &signature,
    "Gity durable merge receipt",
    &tree,
    &parents,
  )?)
}

pub(super) fn observed(
  repo: &Repository,
  input: &MergeOperationInput,
) -> Result<Option<MergeOperationOutcome>> {
  if let Some(outcome) = read(repo, &input.marker()?, input)? {
    return Ok(Some(outcome));
  }
  let Some(MergeOperationOutcome::Applied { commit_sha }) = read(repo, &input.prepared(), input)?
  else {
    return Ok(None);
  };
  let candidate = Oid::from_str(&commit_sha)?;
  let target = match repo.find_reference(&format!("refs/heads/{}", input.target_branch)) {
    Ok(reference) => reference.peel_to_commit()?.id(),
    Err(error) if error.code() == git2::ErrorCode::NotFound => return Ok(None),
    Err(error) => return Err(error.into()),
  };
  if target == candidate || repo.graph_descendant_of(target, candidate)? {
    return Ok(Some(MergeOperationOutcome::Applied { commit_sha }));
  }
  // A crash between prepare and apply, or a subsequent force rewind, cannot
  // safely be classified as rejection. Never guess from the current tip.
  Ok(None)
}
