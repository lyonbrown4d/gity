//! The read side of a shard: eight memory-mapped files plus the structural
//! checks that must pass before any query touches them.
//!
//! `Store::open` maps and parses headers; `Store::verify` walks the small tables
//! (paths, symbols, texts) and the CSR to prove they agree. The occurrence table
//! is not walked here: it is the one table with a million records, so its
//! per-record symbol IDs are bounds-checked when a query reads them.

use crate::csr::{Bits, Refs};
use crate::format::pool_str;
use crate::mapped::{Blob, Table};
use crate::tables::{
  TAG_DOCS, TAG_EXTERNAL, TAG_OCC, TAG_PATHS, TAG_SYMBOLS, TAG_TEXTS, parse_path, parse_symbol,
  parse_text,
};
use crate::{Error, Result};
use std::path::Path;

pub(crate) const PATHS_FILE: &str = "paths.bin";
pub(crate) const OCC_FILE: &str = "occ.bin";
pub(crate) const SYMBOLS_FILE: &str = "symbols.bin";
pub(crate) const EXTERNAL_FILE: &str = "external.bin";
pub(crate) const TEXTS_FILE: &str = "texts.bin";
pub(crate) const DOCS_FILE: &str = "docs.bin";
pub(crate) const REFS_FILE: &str = "refs.bin";
pub(crate) const COVERAGE_FILE: &str = "coverage.bin";

const PATH_RECORD: usize = 20;
const OCC_RECORD: usize = 28;
const SYMBOL_RECORD: usize = 28;
const TEXT_RECORD: usize = 8;

/// Everything a query needs, mapped rather than copied.
#[derive(Debug)]
pub(crate) struct Store {
  pub(crate) paths: Table,
  pub(crate) occ: Table,
  pub(crate) symbols: Table,
  pub(crate) external: Table,
  pub(crate) texts: Table,
  pub(crate) docs: Blob,
  pub(crate) refs: Refs,
  pub(crate) coverage: Bits,
}

impl Store {
  /// Maps every table; a missing or truncated file fails here.
  pub(crate) fn open(dir: &Path) -> Result<Store> {
    let store = Store {
      paths: Table::open(&dir.join(PATHS_FILE), TAG_PATHS, PATHS_FILE, PATH_RECORD)?,
      occ: Table::open(&dir.join(OCC_FILE), TAG_OCC, OCC_FILE, OCC_RECORD)?,
      symbols: Table::open(
        &dir.join(SYMBOLS_FILE),
        TAG_SYMBOLS,
        SYMBOLS_FILE,
        SYMBOL_RECORD,
      )?,
      external: Table::open(
        &dir.join(EXTERNAL_FILE),
        TAG_EXTERNAL,
        EXTERNAL_FILE,
        SYMBOL_RECORD,
      )?,
      texts: Table::open(&dir.join(TEXTS_FILE), TAG_TEXTS, TEXTS_FILE, TEXT_RECORD)?,
      docs: Blob::open(&dir.join(DOCS_FILE), TAG_DOCS, DOCS_FILE)?,
      refs: Refs::open(&dir.join(REFS_FILE))?,
      coverage: Bits::open(&dir.join(COVERAGE_FILE))?,
    };
    store.verify()?;
    Ok(store)
  }

  /// Proves the tables describe the same shard.
  fn verify(&self) -> Result<()> {
    if self.paths.count() != self.coverage.paths() || self.texts.count() != self.paths.count() {
      return Err(Error::Inconsistent);
    }
    if self.refs.symbols() != self.symbols.count()
      || self.refs.last_offset() != Some(self.refs.entries())
    {
      return Err(Error::Inconsistent);
    }
    self.verify_paths()?;
    self.verify_symbols(&self.symbols)?;
    self.verify_symbols(&self.external)?;
    for index in 0..self.texts.count() {
      let record = self
        .texts
        .record(index)
        .and_then(parse_text)
        .ok_or(Error::Corrupt { table: TEXTS_FILE })?;
      if record.text_len > 0
        && pool_str(self.texts.pool(), record.text_off, record.text_len).is_none()
      {
        return Err(Error::Corrupt { table: TEXTS_FILE });
      }
    }
    Ok(())
  }

  /// Paths must cover the occurrence table exactly once, in order.
  fn verify_paths(&self) -> Result<()> {
    let mut end = 0u32;
    for index in 0..self.paths.count() {
      let record = self
        .paths
        .record(index)
        .and_then(parse_path)
        .ok_or(Error::Corrupt { table: PATHS_FILE })?;
      if pool_str(self.paths.pool(), record.text_off, record.text_len).is_none() {
        return Err(Error::Corrupt { table: PATHS_FILE });
      }
      if record.occ_first != end {
        return Err(Error::Inconsistent);
      }
      end = record
        .occ_first
        .checked_add(record.occ_len)
        .ok_or(Error::Inconsistent)?;
      if end > self.occ.count() {
        return Err(Error::Inconsistent);
      }
    }
    if end != self.occ.count() {
      return Err(Error::Inconsistent);
    }
    Ok(())
  }

  /// String pools and documentation ranges of a symbol table.
  fn verify_symbols(&self, table: &Table) -> Result<()> {
    let name = table.table();
    let docs = self.docs.bytes();
    for index in 0..table.count() {
      let record = table
        .record(index)
        .and_then(parse_symbol)
        .ok_or(Error::Corrupt { table: name })?;
      if pool_str(table.pool(), record.text_off, record.text_len).is_none()
        || pool_str(docs, record.doc_off, record.doc_len).is_none()
        || pool_str(docs, record.signature_off, record.signature_len).is_none()
      {
        return Err(Error::Corrupt { table: name });
      }
    }
    Ok(())
  }

  /// First table whose file fell below its mapping.
  pub(crate) fn shrink(&self) -> Option<(&'static str, u64, u64)> {
    let tables: [(&'static str, Option<(u64, u64)>); 8] = [
      (PATHS_FILE, self.paths.shrink()),
      (OCC_FILE, self.occ.shrink()),
      (SYMBOLS_FILE, self.symbols.shrink()),
      (EXTERNAL_FILE, self.external.shrink()),
      (TEXTS_FILE, self.texts.shrink()),
      (DOCS_FILE, self.docs.shrink()),
      (REFS_FILE, self.refs.shrink()),
      (COVERAGE_FILE, self.coverage.shrink()),
    ];
    tables
      .into_iter()
      .find_map(|(name, shrink)| shrink.map(|(mapped, current)| (name, mapped, current)))
  }

  /// Bytes of shard files mapped into this process.
  pub(crate) fn mapped_bytes(&self) -> usize {
    self.paths.mapped_bytes()
      + self.occ.mapped_bytes()
      + self.symbols.mapped_bytes()
      + self.external.mapped_bytes()
      + self.texts.mapped_bytes()
      + self.docs.mapped_bytes()
      + self.refs.mapped_bytes()
      + self.coverage.mapped_bytes()
  }
}
