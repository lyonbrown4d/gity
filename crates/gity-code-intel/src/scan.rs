//! Walks a checked-out tree and parses every supported file.
//!
//! The same scanner runs inside the indexer job (over the job source archive)
//! and in the server-side refresh path, so both produce identical archives.
use crate::Language;
use crate::symbol::SymbolFile;
use anyhow::{Context, Result};
use std::path::Path;

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
      max_total_bytes: 64 * 1024 * 1024,
    }
  }
}

/// Directories that never carry first-party declarations.
const SKIP: [&str; 9] = [
  ".git",
  "target",
  "node_modules",
  "vendor",
  "dist",
  "build",
  ".venv",
  "__pycache__",
  "third_party",
];

/// Result of a scan: the parsed files plus whether a quota stopped it early.
pub struct ScanReport {
  pub files: Vec<SymbolFile>,
  pub truncated: bool,
}

/// Parses every supported file under `root`, applying the size quotas.
pub fn directory(root: &Path, limits: Limits) -> Result<Vec<SymbolFile>> {
  Ok(report(root, limits)?.files)
}

/// Like [`directory`], but tells the caller whether the tree was cut short.
pub fn report(root: &Path, limits: Limits) -> Result<ScanReport> {
  let mut files = Vec::new();
  let mut total = 0usize;
  let mut truncated = false;
  walk(root, root, limits, &mut total, &mut files, &mut truncated)?;
  Ok(ScanReport { files, truncated })
}

fn walk(
  root: &Path,
  directory: &Path,
  limits: Limits,
  total: &mut usize,
  files: &mut Vec<SymbolFile>,
  truncated: &mut bool,
) -> Result<()> {
  let entries = std::fs::read_dir(directory).with_context(|| directory.display().to_string())?;
  let mut paths = entries
    .filter_map(|entry| entry.ok().map(|entry| entry.path()))
    .collect::<Vec<_>>();
  paths.sort();
  for path in paths {
    if files.len() >= limits.max_files || *total >= limits.max_total_bytes {
      *truncated = true;
      return Ok(());
    }
    let metadata = match std::fs::symlink_metadata(&path) {
      Ok(metadata) => metadata,
      Err(_) => continue,
    };
    if metadata.file_type().is_symlink() {
      continue;
    }
    if metadata.is_dir() {
      let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
      if !SKIP.contains(&name) {
        walk(root, &path, limits, total, files, truncated)?;
      }
      continue;
    }
    if metadata.len() as usize > limits.max_file_size {
      *truncated = true;
      continue;
    }
    let relative = path
      .strip_prefix(root)
      .unwrap_or(&path)
      .to_string_lossy()
      .replace('\\', "/");
    let Some(language) = Language::from_path(&relative) else {
      continue;
    };
    let source = match std::fs::read(&path) {
      Ok(bytes) => bytes,
      Err(_) => continue,
    };
    let text = match String::from_utf8(source) {
      Ok(text) => text,
      Err(_) => continue,
    };
    *total += text.len();
    if let Ok(file) = language.parse(&relative, &text) {
      files.push(file);
    }
  }
  Ok(())
}
