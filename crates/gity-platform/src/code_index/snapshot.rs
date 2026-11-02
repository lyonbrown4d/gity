//! Bounded extraction of a Git archive snapshot for server-side scanning.
//!
//! Mirrors the runner's source extraction: Git-archive entries are untrusted
//! input, so parent traversal, `.git` metadata, symlinks and expansion bombs are
//! rejected before anything is written.
use anyhow::{Context, Result, bail};
use std::io::{Read, Seek};
use std::path::Path;

const MAX_ENTRIES: usize = 100_000;
const MAX_EXPANDED_BYTES: u64 = 512 * 1024 * 1024;

pub(crate) fn extract(reader: impl Read + Seek, destination: &Path) -> Result<()> {
  let mut archive = zip::ZipArchive::new(reader)?;
  if archive.len() > MAX_ENTRIES {
    bail!("source archive contains too many entries");
  }
  let mut total = 0u64;
  for index in 0..archive.len() {
    let mut entry = archive.by_index(index)?;
    let enclosed = entry
      .enclosed_name()
      .context("source archive contains an unsafe path")?;
    if enclosed
      .components()
      .any(|part| part.as_os_str().eq_ignore_ascii_case(".git"))
    {
      bail!("source archive contains .git metadata");
    }
    if entry
      .unix_mode()
      .is_some_and(|mode| mode & 0o170000 == 0o120000)
    {
      bail!("source archive contains a symlink");
    }
    total = total
      .checked_add(entry.size())
      .context("source archive size overflow")?;
    if total > MAX_EXPANDED_BYTES {
      bail!("source archive exceeds 512 MiB expanded limit");
    }
    let path = destination.join(enclosed);
    if entry.is_dir() {
      std::fs::create_dir_all(path)?;
      continue;
    }
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
      .write(true)
      .create_new(true)
      .open(&path)?;
    let copied = std::io::copy(&mut entry.by_ref().take(MAX_EXPANDED_BYTES + 1), &mut file)?;
    if copied != entry.size() {
      bail!("source archive entry size mismatch");
    }
  }
  Ok(())
}
