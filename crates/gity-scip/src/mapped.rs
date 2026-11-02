//! Memory-mapped shard tables.
//!
//! Opening a shard maps its files and remembers offsets; a query slices the
//! mapping instead of copying a table into the heap. A shard with a million
//! occurrences therefore costs a handful of file handles and page-table entries
//! per process, not tens of megabytes of resident memory.
//!
//! Mapping is safe here only because of how shards are written: a build writes
//! fresh files and renames them into place, so the bytes behind an existing
//! mapping never change. A file *shrinking* in place (someone truncating it by
//! hand) is the one hazard `mmap` cannot survive, so every table can report that
//! and the query layer refuses to touch the mapping after it happens.

use crate::{Error, Result};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

/// One mapped shard file.
#[derive(Debug)]
pub(crate) struct Mapped {
  file: File,
  mmap: Mmap,
  table: &'static str,
}

impl Mapped {
  /// Maps a table file; an empty or unreadable file is a shard problem, not
  /// an I/O problem the caller can retry.
  pub(crate) fn open(path: &Path, table: &'static str) -> Result<Mapped> {
    let file = std::fs::File::open(path).map_err(|source| Error::Io {
      path: path.display().to_string(),
      source,
    })?;
    let length = file
      .metadata()
      .map_err(|source| Error::Io {
        path: path.display().to_string(),
        source,
      })?
      .len();
    if length < 8 {
      return Err(Error::Corrupt { table });
    }
    // SAFETY: the mapping is read-only and outlives nothing: it is owned by this
    // struct. Shard files are replaced by rename, so the mapped inode is
    // immutable while the mapping exists; a shrink is detected by `shrink`
    // before any query reads through the mapping.
    let mmap = unsafe { Mmap::map(&file) }.map_err(|source| Error::Io {
      path: path.display().to_string(),
      source,
    })?;
    Ok(Mapped { file, mmap, table })
  }

  pub(crate) fn table(&self) -> &'static str {
    self.table
  }

  pub(crate) fn bytes(&self) -> &[u8] {
    &self.mmap
  }

  pub(crate) fn len(&self) -> usize {
    self.mmap.len()
  }

  /// Size of the mapping, i.e. the file bytes reachable through it.
  pub(crate) fn mapped_bytes(&self) -> usize {
    self.mmap.len()
  }

  pub(crate) fn has_tag(&self, tag: [u8; 4]) -> bool {
    self.bytes().get(..4) == Some(&tag)
  }

  pub(crate) fn u32_at(&self, offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let chunk = self.bytes().get(offset..end)?;
    Some(u32::from_le_bytes(chunk.try_into().ok()?))
  }

  /// `(mapped, current)` when the file fell below the mapped length.
  pub(crate) fn shrink(&self) -> Option<(u64, u64)> {
    let mapped = u64::try_from(self.mmap.len()).ok()?;
    let current = self.file.metadata().ok()?.len();
    (current < mapped).then_some((mapped, current))
  }
}

/// One mapped table: an 8-byte header, `count` fixed records, then a pool.
#[derive(Debug)]
pub(crate) struct Table {
  mapped: Mapped,
  count: u32,
  record: usize,
  pool_at: usize,
}

impl Table {
  /// Maps a table and validates that its records fit inside the file.
  pub(crate) fn open(
    path: &Path,
    tag: [u8; 4],
    table: &'static str,
    record: usize,
  ) -> Result<Table> {
    let mapped = Mapped::open(path, table)?;
    if !mapped.has_tag(tag) {
      return Err(Error::Corrupt { table });
    }
    let count = mapped.u32_at(4).ok_or(Error::Corrupt { table })?;
    let records = record
      .checked_mul(count as usize)
      .and_then(|size| size.checked_add(8))
      .ok_or(Error::Corrupt { table })?;
    if records > mapped.len() {
      return Err(Error::Corrupt { table });
    }
    Ok(Table {
      mapped,
      count,
      record,
      pool_at: records,
    })
  }

  pub(crate) fn count(&self) -> u32 {
    self.count
  }

  /// The raw bytes of one record, `None` past the end of the table.
  pub(crate) fn record(&self, index: u32) -> Option<&[u8]> {
    if index >= self.count {
      return None;
    }
    let start = 8usize.checked_add((index as usize).checked_mul(self.record)?)?;
    let end = start.checked_add(self.record)?;
    self.mapped.bytes().get(start..end)
  }

  /// The string pool that follows the records.
  pub(crate) fn pool(&self) -> &[u8] {
    self.mapped.bytes().get(self.pool_at..).unwrap_or(&[])
  }

  pub(crate) fn mapped_bytes(&self) -> usize {
    self.mapped.mapped_bytes()
  }

  pub(crate) fn shrink(&self) -> Option<(u64, u64)> {
    self.mapped.shrink()
  }

  pub(crate) fn table(&self) -> &'static str {
    self.mapped.table()
  }
}

/// A mapped blob table (`docs.bin`): header, length, then the bytes.
#[derive(Debug)]
pub(crate) struct Blob {
  mapped: Mapped,
  length: usize,
}

impl Blob {
  pub(crate) fn open(path: &Path, tag: [u8; 4], table: &'static str) -> Result<Blob> {
    let mapped = Mapped::open(path, table)?;
    if !mapped.has_tag(tag) {
      return Err(Error::Corrupt { table });
    }
    let length = mapped.u32_at(4).ok_or(Error::Corrupt { table })? as usize;
    if length.checked_add(8).is_none_or(|end| end > mapped.len()) {
      return Err(Error::Corrupt { table });
    }
    Ok(Blob { mapped, length })
  }

  pub(crate) fn bytes(&self) -> &[u8] {
    self.mapped.bytes().get(8..8 + self.length).unwrap_or(&[])
  }

  pub(crate) fn mapped_bytes(&self) -> usize {
    self.mapped.mapped_bytes()
  }

  pub(crate) fn shrink(&self) -> Option<(u64, u64)> {
    self.mapped.shrink()
  }
}
