//! Read-side accessors over the shard tables.
//!
//! Every lookup is a binary search or an index into a fixed record, and every
//! one returns `None`/empty instead of failing: a missing answer must let the
//! caller degrade, never break a request.

use crate::assemble::{Raw, Tables};
use crate::format::pool_str;
use crate::model::{Coverage, Location, PositionEncoding, SymbolKind};
use crate::shard::Counts;
use crate::tables::{
  NONE, OccRecord, decode_coverage, decode_docs, decode_occ, decode_paths, decode_refs,
  decode_symbols,
};
use crate::{Error, Result};

impl Tables {
  /// Reads the six tables, rejecting a set that disagrees with itself.
  pub(crate) fn decode(raw: Raw) -> Result<Self> {
    let (paths, path_pool) = decode_paths(&raw.paths)?;
    let occ = decode_occ(&raw.occ)?;
    let (symbols, symbol_pool) = decode_symbols(&raw.symbols)?;
    let docs = decode_docs(&raw.docs)?;
    let (ref_offsets, ref_entries) = decode_refs(&raw.refs)?;
    let (path_count, coverage) = decode_coverage(&raw.coverage)?;
    let tables = Self {
      paths,
      path_pool,
      occ,
      symbols,
      symbol_pool,
      docs,
      ref_offsets,
      ref_entries,
      coverage,
    };
    if !tables.consistent(path_count as usize) {
      return Err(Error::Inconsistent);
    }
    Ok(tables)
  }

  /// Cross-table invariants; a half-written or hand-edited shard fails here.
  fn consistent(&self, path_count: usize) -> bool {
    if path_count != self.paths.len() || self.coverage.len() != path_count.div_ceil(8) {
      return false;
    }
    if self.ref_offsets.len() != self.symbols.len() + 1 {
      return false;
    }
    if self.ref_offsets.last().copied() != Some(self.ref_entries.len() as u32) {
      return false;
    }
    if self.ref_offsets.windows(2).any(|pair| pair[0] > pair[1]) {
      return false;
    }
    let mut end = 0u32;
    for record in &self.paths {
      if record.occ_first != end {
        return false;
      }
      end = record.occ_first.saturating_add(record.occ_len);
      if end as usize > self.occ.len() {
        return false;
      }
      if pool_str(&self.path_pool, record.text_off, record.text_len).is_none() {
        return false;
      }
    }
    if end as usize != self.occ.len() {
      return false;
    }
    if self
      .occ
      .iter()
      .any(|record| record.symbol != NONE && record.symbol as usize >= self.symbols.len())
    {
      return false;
    }
    self
      .symbols
      .iter()
      .all(|record| pool_str(&self.symbol_pool, record.text_off, record.text_len).is_some())
  }

  pub(crate) fn counts(&self) -> Counts {
    Counts {
      paths: self.paths.len(),
      occurrences: self.occ.len(),
      symbols: self.symbols.len(),
      references: self.ref_entries.len(),
      doc_bytes: self.docs.len(),
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

  pub(crate) fn symbol(&self, symbol_id: u32) -> Option<&str> {
    let record = self.symbols.get(symbol_id as usize)?;
    pool_str(&self.symbol_pool, record.text_off, record.text_len)
  }

  pub(crate) fn find_symbol(&self, symbol: &str) -> Option<u32> {
    let found = self.symbols.binary_search_by(|record| {
      pool_str(&self.symbol_pool, record.text_off, record.text_len)
        .unwrap_or("")
        .cmp(symbol)
    });
    found.ok().map(|index| index as u32)
  }

  pub(crate) fn kind(&self, symbol_id: u32) -> SymbolKind {
    self
      .symbols
      .get(symbol_id as usize)
      .map_or(SymbolKind::default(), |record| {
        SymbolKind::from_code(record.kind as i32)
      })
  }

  pub(crate) fn documentation(&self, symbol_id: u32) -> &str {
    self
      .symbols
      .get(symbol_id as usize)
      .and_then(|record| pool_str(&self.docs, record.doc_off, record.doc_len))
      .unwrap_or("")
  }

  pub(crate) fn signature(&self, symbol_id: u32) -> &str {
    self
      .symbols
      .get(symbol_id as usize)
      .and_then(|record| pool_str(&self.docs, record.signature_off, record.signature_len))
      .unwrap_or("")
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

  pub(crate) fn location(&self, occurrence_id: u32) -> Option<Location> {
    let path_id = self.path_of(occurrence_id)?;
    let record = self.occurrence(occurrence_id)?;
    Some(Location {
      path: self.path(path_id)?.to_string(),
      line: record.start_line,
      column: record.start_column,
      end_line: record.end_line,
      end_column: record.end_column,
    })
  }

  /// CSR slice for one symbol: global occurrence IDs in repository order.
  pub(crate) fn occurrences_of_symbol(&self, symbol_id: u32) -> &[u32] {
    let index = symbol_id as usize;
    let Some(start) = self.ref_offsets.get(index).copied() else {
      return &[];
    };
    let Some(end) = self.ref_offsets.get(index + 1).copied() else {
      return &[];
    };
    self
      .ref_entries
      .get(start as usize..end as usize)
      .unwrap_or(&[])
  }
}
