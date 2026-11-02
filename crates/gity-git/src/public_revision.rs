use anyhow::{bail, Result};
use git2::{Commit, Oid, Repository};

/// Public reads and ref creation may name only commits reachable from public
/// branches/tags. Internal receipts and prepared-only candidates remain private,
/// including when addressed by a known SHA, abbreviated SHA, or revspec.
pub(crate) fn resolve<'a>(
  repo: &'a Repository,
  revision: &str,
  default: &str,
) -> Result<Commit<'a>> {
  if revision.starts_with("refs/gity/") || revision == "refs/gity" {
    bail!("git reference not found");
  }
  let candidates = if revision.trim().is_empty() {
    vec![format!("refs/heads/{default}"), "HEAD".to_owned()]
  } else {
    vec![
      format!("refs/heads/{revision}"),
      format!("refs/tags/{revision}"),
      revision.to_owned(),
    ]
  };
  for candidate in candidates {
    if let Ok((object, reference)) = repo.revparse_ext(&candidate) {
      if reference
        .as_ref()
        .and_then(|r| r.name().ok())
        .is_some_and(|name| name.starts_with("refs/gity/"))
      {
        bail!("git reference not found");
      }
      if let Ok(commit) = object.peel_to_commit() {
        if publicly_reachable(repo, commit.id())? {
          return Ok(commit);
        }
        bail!("git reference not found");
      }
    }
  }
  if crate::is_empty(repo)? {
    bail!("git repository is empty");
  }
  bail!("git reference not found")
}

fn publicly_reachable(repo: &Repository, commit: Oid) -> Result<bool> {
  for namespace in ["refs/heads/*", "refs/tags/*"] {
    for reference in repo.references_glob(namespace)? {
      let reference = reference?;
      let Ok(tip) = reference.peel_to_commit() else {
        continue;
      };
      if tip.id() == commit || repo.graph_descendant_of(tip.id(), commit)? {
        return Ok(true);
      }
    }
  }
  Ok(false)
}
