//! Reads the tables back and checks that they agree with each other.
//!
//! `open_strict` surfaces these failures; `open` swallows them so a damaged
//! artifact degrades to the lexical layer instead of breaking a request.

use crate::assemble::{Raw, Tables};
use crate::format::pool_str;
use crate::tables::{
  NONE, decode_coverage, decode_docs, decode_external, decode_occ, decode_paths, decode_refs,
  decode_symbols, decode_texts,
};
use crate::{Error, Result};

impl Tables {
  /// Decodes all eight tables, rejecting a set that disagrees with itself.
  pub(crate) fn decode(raw: Raw) -> Result<Self> {
    let (paths, path_pool) = decode_paths(&raw.paths)?;
    let occ = decode_occ(&raw.occ)?;
    let (symbols, symbol_pool) = decode_symbols(&raw.symbols)?;
    let (external, external_pool) = decode_external(&raw.external)?;
    let (texts, text_pool) = decode_texts(&raw.texts)?;
    let docs = decode_docs(&raw.docs)?;
    let (ref_offsets, ref_entries) = decode_refs(&raw.refs)?;
    let (path_count, coverage) = decode_coverage(&raw.coverage)?;
    let tables = Self {
      paths,
      path_pool,
      occ,
      symbols,
      symbol_pool,
      external,
      external_pool,
      texts,
      text_pool,
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
    if self.texts.len() != path_count {
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
    self.symbols.iter().all(|record| {
      pool_str(&self.symbol_pool, record.text_off, record.text_len).is_some()
        && pool_str(&self.docs, record.doc_off, record.doc_len).is_some()
        && pool_str(&self.docs, record.signature_off, record.signature_len).is_some()
    }) && self.external.iter().all(|record| {
      pool_str(&self.external_pool, record.text_off, record.text_len).is_some()
        && pool_str(&self.docs, record.doc_off, record.doc_len).is_some()
    }) && self.texts.iter().all(|record| {
      record.text_len == 0 || pool_str(&self.text_pool, record.text_off, record.text_len).is_some()
    })
  }
}
