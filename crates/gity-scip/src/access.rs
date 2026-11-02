//! Path-side accessors: coverage, encoding, stored text, occurrence ranges.
//!
//! Every lookup is a binary search or an index into a fixed record, and every
//! one returns `None`/empty instead of failing: a missing answer must let the
//! caller degrade, never break a request.

use crate::answer::{Coverage, Location};
use crate::assemble::Tables;
use crate::format::pool_str;
use crate::manifest::Counts;
use crate::model::PositionEncoding;
use crate::tables::OccRecord;

impl Tables {
  pub(crate) fn counts(&self) -> Counts {
    Counts {
      paths: self.paths.len(),
      occurrences: self.occ.len(),
      symbols: self.symbols.len(),
      external_symbols: self.external.len(),
      references: self.ref_entries.len(),
      doc_bytes: self.docs.len(),
      text_bytes: self.text_pool.len(),
    }
  }

  pub(crate) fn coverage_summary(&self) -> (usize, usize) {
    let precise = (0..self.paths.len())
      .filter(|id| self.is_precise(*id as u32))
      .count();
    (precise, self.paths.len() - precise)
  }

  pub(crate) fn is_precise(&self, path_id: u32) -> bool {
    let index = path_id as usize;
    self
      .coverage
      .get(index / 8)
      .is_some_and(|byte| byte & (0x80 >> (index % 8)) != 0)
  }

  pub(crate) fn path(&self, path_id: u32) -> Option<&str> {
    let record = self.paths.get(path_id as usize)?;
    pool_str(&self.path_pool, record.text_off, record.text_len)
  }

  pub(crate) fn find_path(&self, path: &str) -> Option<u32> {
    let found = self.paths.binary_search_by(|record| {
      pool_str(&self.path_pool, record.text_off, record.text_len)
        .unwrap_or("")
        .cmp(path)
    });
    found.ok().map(|index| index as u32)
  }

  pub(crate) fn coverage(&self, path: &str) -> Coverage {
    match self.find_path(path) {
      None => Coverage::Unknown,
      Some(id) if self.is_precise(id) => Coverage::Precise,
      Some(_) => Coverage::Listed,
    }
  }

  pub(crate) fn encoding(&self, path_id: u32) -> Option<PositionEncoding> {
    self
      .paths
      .get(path_id as usize)
      .map(|record| PositionEncoding::from_code(record.encoding as i32))
  }

  /// Source text the indexer shipped with a document, when it shipped any.
  pub(crate) fn text(&self, path_id: u32) -> Option<&str> {
    let record = self.texts.get(path_id as usize)?;
    if record.text_len == 0 {
      return None;
    }
    pool_str(&self.text_pool, record.text_off, record.text_len)
  }

  /// First global occurrence ID owned by a path.
  pub(crate) fn occurrence_base(&self, path_id: u32) -> u32 {
    self
      .paths
      .get(path_id as usize)
      .map_or(0, |record| record.occ_first)
  }

  /// Occurrence records owned by one path, empty for an uncovered path.
  pub(crate) fn occurrences(&self, path_id: u32) -> &[OccRecord] {
    let Some(record) = self.paths.get(path_id as usize) else {
      return &[];
    };
    let start = record.occ_first as usize;
    let end = start.saturating_add(record.occ_len as usize);
    self.occ.get(start..end).unwrap_or(&[])
  }

  /// Path owning a global occurrence ID, using the contiguous ranges.
  pub(crate) fn path_of(&self, occurrence_id: u32) -> Option<u32> {
    let index = self
      .paths
      .partition_point(|record| record.occ_first <= occurrence_id);
    let record = self.paths.get(index.checked_sub(1)?)?;
    if occurrence_id < record.occ_first.saturating_add(record.occ_len) {
      return Some(index as u32 - 1);
    }
    None
  }

  pub(crate) fn occurrence(&self, occurrence_id: u32) -> Option<&OccRecord> {
    self.occ.get(occurrence_id as usize)
  }

  /// In-repository position of an occurrence; `external` stays false here
  /// because an occurrence always lives in one of our files.
  pub(crate) fn location(&self, occurrence_id: u32) -> Option<Location> {
    let path_id = self.path_of(occurrence_id)?;
    let record = self.occurrence(occurrence_id)?;
    Some(Location {
      path: self.path(path_id)?.to_string(),
      line: record.start_line,
      column: record.start_column,
      end_line: record.end_line,
      end_column: record.end_column,
      external: false,
    })
  }
}
