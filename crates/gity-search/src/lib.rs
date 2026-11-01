//! Persistent, commit-pinned candidate indexes. Live Git matching remains the
//! authoritative contract for line numbers, case, Unicode, and substring results.
mod build;
mod query;
mod schema;
use anyhow::{Context, Result, bail};
use fs2::FileExt;
use gity_git::GitService;
use serde::{Deserialize, Serialize};
use std::{
  fs::File,
  path::{Path, PathBuf},
  sync::Arc,
};

#[derive(Clone, Copy, Debug)]
pub struct Limits {
  pub max_files: usize,
  pub max_file_size: usize,
  pub max_total_bytes: usize,
}
impl Default for Limits {
  fn default() -> Self {
    Self {
      max_files: 5000,
      max_file_size: 512 * 1024,
      max_total_bytes: 256 * 1024 * 1024,
    }
  }
}
#[derive(Clone)]
pub struct SearchIndex {
  root: Arc<PathBuf>,
  git: GitService,
  limits: Limits,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexStatus {
  pub revision: String,
  pub indexed_files: usize,
  pub total_bytes: usize,
  pub truncated: bool,
  #[serde(default)]
  pub unchanged: bool,
}
#[derive(Serialize, Deserialize)]
struct Metadata {
  version: u32,
  status: IndexStatus,
  max_files: usize,
  max_file_size: usize,
  max_total_bytes: usize,
}
impl SearchIndex {
  pub fn new(root: impl Into<PathBuf>, git: GitService, limits: Limits) -> Result<Self> {
    if limits.max_files == 0
      || limits.max_files > 100_000
      || limits.max_file_size == 0
      || limits.max_file_size > 10 * 1024 * 1024
      || limits.max_total_bytes == 0
    {
      bail!("invalid search index limits");
    }
    let root = root.into();
    std::fs::create_dir_all(&root)?;
    Ok(Self {
      root: Arc::new(root.canonicalize()?),
      git,
      limits,
    })
  }
  fn path(&self, project_id: i64) -> Result<PathBuf> {
    if project_id <= 0 {
      bail!("positive project ID required");
    }
    let path = self.root.join(project_id.to_string());
    if std::fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
      bail!("search index path must not be a symlink");
    }
    if std::fs::symlink_metadata(&path).is_ok()
      && !path.canonicalize()?.starts_with(self.root.as_path())
    {
      bail!("search index path escapes root");
    }
    Ok(path)
  }
  fn lock(&self, project_id: i64, exclusive: bool) -> Result<Option<File>> {
    self.path(project_id)?;
    let path = self.root.join(format!("{project_id}.lock"));
    if std::fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
      bail!("search index lock must not be a symlink");
    }
    if std::fs::symlink_metadata(&path).is_ok()
      && !path.canonicalize()?.starts_with(self.root.as_path())
    {
      bail!("search index lock escapes root");
    }
    let file = std::fs::OpenOptions::new()
      .create(true)
      .truncate(false)
      .read(true)
      .write(true)
      .open(path)?;
    let result = if exclusive {
      FileExt::try_lock_exclusive(&file)
    } else {
      FileExt::try_lock_shared(&file)
    };
    match result {
      Ok(()) => Ok(Some(file)),
      Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
      Err(error) => Err(error.into()),
    }
  }
  pub async fn delete(&self, project_id: i64) -> Result<()> {
    let index = self.clone();
    tokio::task::spawn_blocking(move || {
      let _lock = index
        .lock(project_id, true)?
        .context("search index is being used; retry deletion")?;
      let path = index.path(project_id)?;
      if path.exists() {
        std::fs::remove_dir_all(path)?;
      }
      Ok(())
    })
    .await?
  }
}
fn metadata(index: &tantivy::Index) -> Result<Option<Metadata>> {
  let metadata = index.load_metas()?;
  metadata
    .payload
    .map(|payload| serde_json::from_str(&payload).map_err(Into::into))
    .transpose()
}
fn same_limits(metadata: &Metadata, limits: Limits) -> bool {
  metadata.version == 1
    && metadata.max_files == limits.max_files
    && metadata.max_file_size == limits.max_file_size
    && metadata.max_total_bytes == limits.max_total_bytes
}
fn open(path: &Path) -> Result<tantivy::Index> {
  let index = tantivy::Index::open_in_dir(path)?;
  schema::register(&index)?;
  Ok(index)
}
