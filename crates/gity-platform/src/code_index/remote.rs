//! Object-storage copy of code-index archives, shared by every replica.
//!
//! The runner keeps uploading through the CI artifact channel; after a live
//! ingest or a manual rebuild this module mirrors the archive to
//! `code-index/{project_id}/{commit}.msgpack.zst`. A replica whose local store
//! has no snapshot for that commit can fetch the object, decode it and refresh
//! its own index, which is what lets several API processes serve the same code
//! intelligence.
//!
//! Every operation here is best effort: object storage being down must never
//! fail an ingest, a rebuild or a read. Failures are logged and counted, and the
//! caller keeps whatever the local store already has.

use crate::AppState;
use bytes::Bytes;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

/// Key prefix shared with other object-storage users (`ci/...`, `packages/...`).
pub(crate) const PREFIX: &str = "code-index";
pub(crate) const EXTENSION: &str = "msgpack.zst";

/// Object reads performed by this process, for tests that assert short-circuit.
#[cfg(test)]
static REMOTE_READS: AtomicUsize = AtomicUsize::new(0);

#[cfg(test)]
pub(crate) fn remote_reads() -> usize {
  REMOTE_READS.load(Ordering::SeqCst)
}

fn count_read() {
  #[cfg(test)]
  REMOTE_READS.fetch_add(1, Ordering::SeqCst);
}

/// Object key for one archived commit, `None` when the inputs cannot form one.
///
/// Commits are hex object names, so anything else is a bug in the caller rather
/// than a repository state; refusing keeps the key space clean.
pub(crate) fn archive_key(project_id: i64, commit: &str) -> Option<String> {
  let valid = project_id > 0
    && !commit.is_empty()
    && commit.len() <= 64
    && commit.bytes().all(|byte| byte.is_ascii_hexdigit());
  valid.then(|| format!("{PREFIX}/{project_id}/{commit}.{EXTENSION}"))
}

fn enabled(s: &AppState) -> bool {
  s.settings.search.code_index_remote
}

/// Copies archive bytes to object storage; never returns an error.
pub(crate) async fn publish(s: &AppState, project_id: i64, commit: &str, bytes: Bytes) {
  if !enabled(s) {
    return;
  }
  let Some(key) = archive_key(project_id, commit) else {
    count("publish", "failure");
    tracing::warn!(project_id, commit, "code index commit cannot be archived");
    return;
  };
  match s.storage.put(&key, bytes).await {
    Ok(()) => {
      count("publish", "success");
      tracing::debug!(project_id, commit, %key, "code index archive mirrored");
    }
    Err(error) => {
      count("publish", "failure");
      tracing::warn!(error = %error, project_id, commit, "code index archive mirror failed");
    }
  }
}

/// Encodes an in-memory archive and publishes it; never returns an error.
pub(crate) async fn publish_archive(
  s: &AppState,
  project_id: i64,
  archive: &gity_code_intel::Archive,
) {
  if !enabled(s) {
    return;
  }
  match gity_code_intel::encode(archive) {
    Ok(bytes) => publish(s, project_id, &archive.commit, Bytes::from(bytes)).await,
    Err(error) => {
      count("publish", "failure");
      tracing::warn!(error = %error, project_id, "code index archive encoding failed");
    }
  }
}

/// Reads and decodes one archive; `None` covers every failure including a miss.
pub(crate) async fn fetch(
  s: &AppState,
  project_id: i64,
  commit: &str,
) -> Option<gity_code_intel::Archive> {
  if !enabled(s) {
    return None;
  }
  let key = archive_key(project_id, commit)?;
  count_read();
  let bytes = match s.storage.get(&key).await {
    Ok(bytes) => bytes,
    Err(error) if error.status.as_u16() == 404 => {
      count("fetch", "missing");
      tracing::debug!(project_id, commit, %key, "no shared code index archive");
      return None;
    }
    Err(error) => {
      count("fetch", "failure");
      tracing::warn!(error = %error, project_id, commit, "code index archive read failed");
      return None;
    }
  };
  match super::blocking({
    let bytes = bytes.clone();
    move || gity_code_intel::decode(&bytes)
  })
  .await
  {
    Ok(archive) => {
      count("fetch", "success");
      tracing::info!(
        project_id,
        commit,
        bytes = bytes.len(),
        "code index archive restored from object storage"
      );
      Some(archive)
    }
    Err(error) => {
      count("fetch", "failure");
      tracing::warn!(error = %error, project_id, commit, "shared code index archive is unreadable");
      None
    }
  }
}

/// Makes the local store serve `commit`, restoring it from object storage when
/// the local snapshot is missing or from a different commit.
///
/// Returns whether the local store now serves that commit. A matching local
/// snapshot short-circuits before any object-storage call, so the common case
/// costs one status read.
pub(crate) async fn hydrate(s: &AppState, project_id: i64, commit: &str) -> bool {
  if !enabled(s) {
    return false;
  }
  if super::local_status(s, project_id)
    .await
    .is_ok_and(|status| status.is_some_and(|status| status.commit == commit))
  {
    count("hydrate", "local");
    return true;
  }
  let Some(archive) = fetch(s, project_id, commit).await else {
    count("hydrate", "missing");
    return false;
  };
  let store = s.code_index.clone();
  match super::blocking(move || store.refresh(project_id, &archive)).await {
    Ok(_) => {
      count("hydrate", "success");
      true
    }
    Err(error) => {
      count("hydrate", "failure");
      tracing::warn!(error = %error, project_id, commit, "restored code index could not be written");
      false
    }
  }
}

/// Restores the local index for a project's default-branch HEAD when the shared
/// copy holds it.
///
/// This is the entry point a read path needs: it resolves the commit that the
/// index job would have used, then delegates to [`hydrate`]. A missing git
/// checkout or repository simply answers `false`.
pub(crate) async fn hydrate_head(s: &AppState, project: &serde_json::Value) -> bool {
  let Ok(project_id) = super::project_id(project) else {
    return false;
  };
  let repo = crate::repository::repo(project);
  let branch = crate::repository::default_branch(project).to_owned();
  let Ok(commit) = s.git.resolve_revision(&repo, &branch, "").await else {
    count("hydrate", "missing");
    return false;
  };
  hydrate(s, project_id, &commit).await
}

fn count(op: &'static str, result: &'static str) {
  metrics::counter!("gity_code_index_remote_total", "op" => op, "result" => result).increment(1);
}
