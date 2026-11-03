//! File-based store: one directory per project holding the query index, the
//! archive of the indexed commit, and a small status document.
//!
//! Layout mirrors the search index so operators only learn one convention:
//!
//! ```text
//! <root>/<project_id>/current/          tantivy index of the newest commit
//! <root>/<project_id>/archive/<commit>.msgpack.zst
//! <root>/<project_id>/status.json       commit, counts, timestamps
//! <root>/<project_id>.lock              writer/reader coordination
//! ```
use crate::archive::{self, Archive};
use crate::index::{self, SymbolDoc};
use crate::scan::{self, Limits};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
  pub commit: String,
  pub files: usize,
  pub definitions: usize,
  pub archive_bytes: usize,
  pub indexed_at: String,
  /// True when the index covers only part of the tree because a quota was hit.
  #[serde(default)]
  pub truncated: bool,
}

#[derive(Clone)]
pub struct Store {
  root: PathBuf,
}

impl Store {
  pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
    let root = root.into();
    std::fs::create_dir_all(&root).with_context(|| root.display().to_string())?;
    Ok(Self {
      root: root.canonicalize().context("resolve code index root")?,
    })
  }

  fn project(&self, project: i64) -> Result<PathBuf> {
    if project <= 0 {
      bail!("positive project ID required");
    }
    let path = self.root.join(project.to_string());
    if std::fs::symlink_metadata(&path).is_ok_and(|meta| meta.file_type().is_symlink()) {
      bail!("code index path must not be a symlink");
    }
    Ok(path)
  }

  /// Shared locks degrade to `None` when a writer holds the index, so a busy
  /// rebuild reads as "no index yet" instead of failing the request.
  fn lock(&self, project: i64, exclusive: bool) -> Result<Option<File>> {
    let path = self.root.join(format!("{project}.lock"));
    std::fs::OpenOptions::new()
      .create(true)
      .truncate(false)
      .read(true)
      .write(true)
      .open(path)
      .with_context(|| format!("open code index lock for project {project}"))
      .and_then(|file| {
        // Disambiguate from std's inherent File methods, which return a bool.
        let outcome = if exclusive {
          fs2::FileExt::try_lock_exclusive(&file)
        } else {
          fs2::FileExt::try_lock_shared(&file)
        };
        match outcome {
          Ok(()) => Ok(Some(file)),
          Err(error) if exclusive => {
            Err(error).with_context(|| format!("code index for project {project} is busy"))
          }
          // A reader that cannot lock simply reports "no index yet".
          Err(_) => Ok(None),
        }
      })
  }

  /// Indexes an in-memory archive, replacing the previous snapshot.
  pub fn refresh(&self, project: i64, archive: &Archive) -> Result<Status> {
    let directory = self.project(project)?;
    self.lock(project, true)?;
    std::fs::create_dir_all(directory.join("archive"))?;
    let bytes = archive::encode(archive)?;
    let archive_path = directory
      .join("archive")
      .join(format!("{}.msgpack.zst", archive.commit));
    std::fs::write(&archive_path, &bytes).context("write symbol archive")?;
    let index_dir = directory.join("current");
    let definitions = index::build(&index_dir, &archive.files)?;
    let status = Status {
      commit: archive.commit.clone(),
      files: archive.files.len(),
      definitions,
      archive_bytes: bytes.len(),
      truncated: archive.truncated,
      indexed_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    };
    let text = serde_json::to_string_pretty(&status)?;
    std::fs::write(directory.join("status.json"), text).context("write index status")?;
    prune(&directory, 10)?;
    Ok(status)
  }

  /// Scans a checkout and indexes it in one step.
  pub fn refresh_directory(
    &self,
    project: i64,
    commit: &str,
    checkout: &Path,
    limits: Limits,
  ) -> Result<Status> {
    let report = scan::report(checkout, limits)?;
    self.refresh(
      project,
      &Archive::new(commit, report.files).truncated(report.truncated),
    )
  }

  pub fn status(&self, project: i64) -> Result<Option<Status>> {
    if self.lock(project, false)?.is_none() {
      return Ok(None);
    }
    read_status(&self.project(project)?)
  }

  pub fn symbols(&self, project: i64, path: &str) -> Result<Vec<SymbolDoc>> {
    if self.lock(project, false)?.is_none() {
      return Ok(Vec::new());
    }
    let directory = self.project(project)?;
    let Some((index, fields)) = index::open(&directory.join("current"))? else {
      return Ok(Vec::new());
    };
    index::symbols_in(&index, &fields, path)
  }

  /// Innermost declaration covering a position in a file.
  pub fn definition_at(
    &self,
    project: i64,
    path: &str,
    line: u64,
    column: u64,
  ) -> Result<Option<SymbolDoc>> {
    let mut rows = self.symbols(project, path)?;
    rows.retain(|row| row.line <= line && line <= row.line_end.max(row.line));
    // Innermost wins: the latest start line, preferring a column at or before
    // the cursor so `Foo::bar` resolves to `bar` rather than `Foo`.
    rows.sort_by_key(|row| (std::cmp::Reverse(row.line), row.column > column));
    Ok(rows.into_iter().next())
  }

  pub fn hover_at(
    &self,
    project: i64,
    path: &str,
    line: u64,
    column: u64,
  ) -> Result<Option<SymbolDoc>> {
    let definition = self.definition_at(project, path, line, column)?;
    Ok(definition.filter(|row| !row.doc.is_empty() || !row.name.is_empty()))
  }

  /// Declarations of a name across the indexed snapshot.
  pub fn references(&self, project: i64, name: &str) -> Result<Vec<SymbolDoc>> {
    if self.lock(project, false)?.is_none() {
      return Ok(Vec::new());
    }
    let directory = self.project(project)?;
    let Some((index, fields)) = index::open(&directory.join("current"))? else {
      return Ok(Vec::new());
    };
    index::by_name(&index, &fields, name)
  }

  /// Free-text lookup over declaration names, used by the outline search box.
  pub fn search(&self, project: i64, text: &str, limit: usize) -> Result<Vec<SymbolDoc>> {
    if self.lock(project, false)?.is_none() {
      return Ok(Vec::new());
    }
    let directory = self.project(project)?;
    let Some((index, fields)) = index::open(&directory.join("current"))? else {
      return Ok(Vec::new());
    };
    index::search(&index, &fields, text, limit)
  }

  pub fn forget(&self, project: i64) -> Result<()> {
    self.lock(project, true)?;
    let directory = self.project(project)?;
    if directory.exists() {
      std::fs::remove_dir_all(directory).context("remove code index")?;
    }
    Ok(())
  }
}

fn read_status(directory: &Path) -> Result<Option<Status>> {
  let path = directory.join("status.json");
  let Ok(text) = std::fs::read_to_string(&path) else {
    return Ok(None);
  };
  Ok(serde_json::from_str(&text).ok())
}

/// Keeps the newest archives so historical commits stay jumpable for a while.
fn prune(directory: &Path, keep: usize) -> Result<()> {
  let archive_dir = directory.join("archive");
  let mut entries = std::fs::read_dir(&archive_dir)?
    .filter_map(|entry| entry.ok())
    .map(|entry| entry.path())
    .collect::<Vec<_>>();
  if entries.len() <= keep {
    return Ok(());
  }
  entries.sort_by_key(|path| {
    std::fs::metadata(path)
      .and_then(|meta| meta.modified())
      .ok()
  });
  let excess = entries.len().saturating_sub(keep);
  for path in entries.into_iter().take(excess) {
    let _ = std::fs::remove_file(path);
  }
  Ok(())
}
