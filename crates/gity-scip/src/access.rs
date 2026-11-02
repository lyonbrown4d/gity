//! Path-side accessors over the mapped tables: coverage, encoding, stored text
//! and occurrence ranges.
//!
//! Every lookup is a binary search over mapped records or one record read by
//! offset, and every one returns `None`/empty instead of failing: a missing
//! answer must let the caller degrade, never break a request.

use crate::answer::{Coverage, Location};
use crate::format::pool_str;
use crate::model::PositionEncoding;
use crate::read::Store;
use crate::tables::{OccRecord, parse_occ, parse_path, parse_text};

impl Store {
  pub(crate) fn is_precise(&self, path_id: u32) -> bool {
    self.coverage.contains(path_id)
  }

  pub(crate) fn path(&self, path_id: u32) -> Option<&str> {
    let record = parse_path(self.paths.record(path_id)?)?;
    pool_str(self.paths.pool(), record.text_off, record.text_len)
  }

  pub(crate) fn find_path(&self, path: &str) -> Option<u32> {
    let mut low = 0u32;
    let mut high = self.paths.count();
    while low < high {
      let middle = low + (high - low) / 2;
      match self.path(middle)?.cmp(path) {
        std::cmp::Ordering::Less => low = middle + 1,
        std::cmp::Ordering::Greater => high = middle,
        std::cmp::Ordering::Equal => return Some(middle),
      }
    }
    None
  }

  pub(crate) fn coverage(&self, path: &str) -> Coverage {
    match self.find_path(path) {
      None => Coverage::Unknown,
      Some(id) if self.is_precise(id) => Coverage::Precise,
      Some(_) => Coverage::Listed,
    }
  }

  pub(crate) fn encoding(&self, path_id: u32) -> Option<PositionEncoding> {
    let record = parse_path(self.paths.record(path_id)?)?;
    Some(PositionEncoding::from_code(record.encoding as i32))
  }

  /// Source text the indexer shipped with a document, when it shipped any.
  pub(crate) fn text(&self, path_id: u32) -> Option<&str> {
    let record = parse_text(self.texts.record(path_id)?)?;
    if record.text_len == 0 {
      return None;
    }
    pool_str(self.texts.pool(), record.text_off, record.text_len)
  }

  /// First global occurrence ID owned by a path.
  pub(crate) fn occurrence_base(&self, path_id: u32) -> u32 {
    parse_path(self.paths.record(path_id).unwrap_or(&[])).map_or(0, |record| record.occ_first)
  }

  /// Occurrence records owned by one path, as a view over the mapping.
  pub(crate) fn occurrences(&self, path_id: u32) -> PathOccurrences<'_> {
    let Some(record) = self.paths.record(path_id).and_then(parse_path) else {
      return PathOccurrences {
        store: self,
        first: 0,
        count: 0,
      };
    };
    PathOccurrences {
      store: self,
      first: record.occ_first,
      count: record.occ_len,
    }
  }

  /// Path owning a global occurrence ID, using the contiguous ranges.
  pub(crate) fn path_of(&self, occurrence_id: u32) -> Option<u32> {
    let mut low = 0u32;
    let mut high = self.paths.count();
    while low < high {
      let middle = low + (high - low) / 2;
      let record = parse_path(self.paths.record(middle)?)?;
      if record.occ_first <= occurrence_id {
        low = middle + 1;
      } else {
        high = middle;
      }
    }
    let candidate = low.checked_sub(1)?;
    let record = parse_path(self.paths.record(candidate)?)?;
    if occurrence_id < record.occ_first.saturating_add(record.occ_len) {
      return Some(candidate);
    }
    None
  }

  pub(crate) fn occurrence(&self, occurrence_id: u32) -> Option<OccRecord> {
    parse_occ(self.occ.record(occurrence_id)?)
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

/// One path's slice of the occurrence table, decoded on demand.
pub(crate) struct PathOccurrences<'a> {
  store: &'a Store,
  first: u32,
  count: u32,
}

impl PathOccurrences<'_> {
  pub(crate) fn len(&self) -> u32 {
    self.count
  }

  pub(crate) fn get(&self, index: u32) -> Option<OccRecord> {
    if index >= self.count {
      return None;
    }
    parse_occ(self.store.occ.record(self.first.checked_add(index)?)?)
  }

  /// Index of the first record for which `predicate` is false; the records are
  /// ordered by position, so this is a binary search over mapped records.
  pub(crate) fn partition_point(&self, mut predicate: impl FnMut(OccRecord) -> bool) -> u32 {
    let mut low = 0u32;
    let mut high = self.count;
    while low < high {
      let middle = low + (high - low) / 2;
      let Some(record) = self.get(middle) else {
        return low;
      };
      if predicate(record) {
        low = middle + 1;
      } else {
        high = middle;
      }
    }
    low
  }
}
