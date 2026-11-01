use crate::mutations::{signature, validate_ref};
use crate::{GitService, MergeBranchesInput};
use anyhow::{bail, Result};
use serde_json::{json, Value};
impl GitService {
  pub async fn merge_branches(&self, repo: &str, input: MergeBranchesInput) -> Result<()> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::MergeBranches {
            input: serde_json::to_value(&input)?,
          },
        )
        .await;
    }
    let target = validate_ref(&input.target_branch, "heads")?;
    let source = validate_ref(&input.source_branch, "heads")?;
    self
      .read(repo, move |r| {
        // Lock both tips across validation and object construction. A native
        // push cannot advance the checked source between CI approval and merge.
        let mut transaction = r.transaction()?;
        let mut references = vec![source.as_str(), target.as_str()];
        references.sort_unstable();
        references.dedup();
        for reference in references {
          transaction.lock_ref(reference)?;
        }
        let target_commit = r.find_reference(&target)?.peel_to_commit()?;
        let source_commit = r.find_reference(&source)?.peel_to_commit()?;
        if let Some(expected) = &input.expected_source_sha {
          if expected != &source_commit.id().to_string() {
            bail!("merge source changed; refresh required");
          }
        }
        if target_commit.id() == source_commit.id()
          || r.graph_descendant_of(target_commit.id(), source_commit.id())?
        {
          return Ok(Value::Null);
        }
        let mut index = r.merge_commits(&target_commit, &source_commit, None)?;
        if index.has_conflicts() {
          bail!("git merge conflict");
        }
        let tree = r.find_tree(index.write_tree_to(&r)?)?;
        let signature = signature(&input.author_name, &input.author_email)?;
        let message = if input.message.trim().is_empty() {
          format!(
            "Merge branch '{}' into '{}'",
            input.source_branch, input.target_branch
          )
        } else {
          input.message
        };
        let id = r.commit(
          None,
          &signature,
          &signature,
          &message,
          &tree,
          &[&target_commit, &source_commit],
        )?;
        transaction.set_target(&target, id, Some(&signature), "Gity merge")?;
        transaction.commit()?;
        Ok(Value::Null)
      })
      .await?;
    Ok(())
  }

  pub async fn diff_branches(&self, repo: &str, target: &str, source: &str) -> Result<String> {
    if self.remote.is_some() {
      return self
        .execute_remote(
          repo,
          gity_git_wire::GitOperation::DiffBranches {
            target: target.into(),
            source: source.into(),
          },
        )
        .await;
    }
    let target = validate_ref(target, "heads")?;
    let source = validate_ref(source, "heads")?;
    let value = self
      .read(repo, move |r| {
        let t = r.find_reference(&target)?.peel_to_commit()?;
        let s = r.find_reference(&source)?.peel_to_commit()?;
        let base = r.find_commit(r.merge_base(t.id(), s.id())?)?;
        let mut diff = r.diff_tree_to_tree(Some(&base.tree()?), Some(&s.tree()?), None)?;
        diff.find_similar(None)?;
        let mut bytes = Vec::new();
        diff.print(git2::DiffFormat::Patch, |_, _, line| {
          if matches!(line.origin(), ' ' | '+' | '-') {
            bytes.push(line.origin() as u8);
          }
          bytes.extend_from_slice(line.content());
          true
        })?;
        Ok(json!(String::from_utf8_lossy(&bytes)))
      })
      .await?;
    Ok(value.as_str().unwrap_or_default().to_owned())
  }
}
