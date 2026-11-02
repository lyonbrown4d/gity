//! Builds shard tables from a decoded index.
//!
//! Ordering rules the query layer relies on:
//!
//! - paths are sorted by byte order, so path lookup is a binary search;
//! - each path owns a contiguous occurrence range, sorted by position, so a
//!   position lookup is another binary search;
//! - symbols are sorted by their SCIP string, so symbol lookup is a binary
//!   search, and occurrence IDs are assigned in path order so CSR entries for
//!   one symbol come out in repository order;
//! - text records are appended in path order, so a path ID indexes its text.

use crate::model::{Decoded, DecodedDocument, PositionEncoding};
use crate::symbols;
use crate::tables::{NONE, OccRecord, PathRecord, SymbolRecord, TextRecord};
use crate::{Error, Result};
use std::collections::{BTreeMap, BTreeSet};

/// Raw table bytes as they arrive from disk.
pub(crate) struct Raw {
  pub paths: Vec<u8>,
  pub occ: Vec<u8>,
  pub symbols: Vec<u8>,
  pub external: Vec<u8>,
  pub texts: Vec<u8>,
  pub docs: Vec<u8>,
  pub refs: Vec<u8>,
  pub coverage: Vec<u8>,
}

/// The shard's in-memory tables; queries never touch the filesystem.
#[derive(Clone, Debug, Default)]
pub(crate) struct Tables {
  pub paths: Vec<PathRecord>,
  pub path_pool: Vec<u8>,
  pub occ: Vec<OccRecord>,
  pub symbols: Vec<SymbolRecord>,
  pub symbol_pool: Vec<u8>,
  pub external: Vec<SymbolRecord>,
  pub external_pool: Vec<u8>,
  pub texts: Vec<TextRecord>,
  pub text_pool: Vec<u8>,
  pub docs: Vec<u8>,
  pub ref_offsets: Vec<u32>,
  pub ref_entries: Vec<u32>,
  pub coverage: Vec<u8>,
}

impl Tables {
  /// Assigns IDs, orders records and computes the CSR from a decoded index.
  pub(crate) fn from_decoded(decoded: &Decoded) -> Result<Self> {
    let mut precise: BTreeMap<String, &DecodedDocument> = BTreeMap::new();
    for document in &decoded.documents {
      let path = normalize(&document.path)?;
      if precise.insert(path.clone(), document).is_some() {
        return Err(Error::DuplicatePath(path));
      }
    }
    let mut listed = BTreeSet::new();
    for path in &decoded.uncovered_paths {
      let path = normalize(path)?;
      if !precise.contains_key(&path) {
        listed.insert(path);
      }
    }
    let mut symbol_pool = Vec::new();
    let mut docs = Vec::new();
    let (symbols, ids) = symbols::assign(
      symbols::document_aggregates(precise.values().copied()),
      &mut symbol_pool,
      &mut docs,
    );
    let mut external_pool = Vec::new();
    let (external, _) = symbols::assign(
      symbols::external_aggregates(&decoded.external),
      &mut external_pool,
      &mut docs,
    );
    let mut tables = Self {
      symbols,
      symbol_pool,
      external,
      external_pool,
      docs,
      ..Self::default()
    };
    tables.assemble_paths(&precise, &listed, &ids);
    tables.assemble_refs();
    Ok(tables)
  }

  fn assemble_paths(
    &mut self,
    precise: &BTreeMap<String, &DecodedDocument>,
    listed: &BTreeSet<String>,
    ids: &BTreeMap<&str, u32>,
  ) {
    let mut names: BTreeSet<&str> = precise.keys().map(String::as_str).collect();
    names.extend(listed.iter().map(String::as_str));
    for name in names {
      let document = precise.get(name).copied();
      let occ_first = self.occ.len() as u32;
      if let Some(document) = document {
        let mut records: Vec<OccRecord> = document
          .occurrences
          .iter()
          .map(|occurrence| OccRecord {
            start_line: occurrence.span.start_line,
            start_column: occurrence.span.start_column,
            end_line: occurrence.span.end_line,
            end_column: occurrence.span.end_column,
            symbol: ids.get(occurrence.symbol.as_str()).copied().unwrap_or(NONE),
            roles: occurrence.roles.bits(),
            enclosing: NONE,
          })
          .collect();
        records.sort();
        symbols::fill_enclosing(&mut records);
        self.occ.extend_from_slice(&records);
      }
      let text = document.map_or("", |document| document.text.as_str());
      let text_offset = self.text_pool.len() as u32;
      self.text_pool.extend_from_slice(text.as_bytes());
      self.texts.push(TextRecord {
        text_off: text_offset,
        text_len: text.len() as u32,
      });
      let text_off = self.path_pool.len() as u32;
      self.path_pool.extend_from_slice(name.as_bytes());
      let encoding = document.map_or(PositionEncoding::Unspecified, |document| {
        document.position_encoding
      });
      self.paths.push(PathRecord {
        text_off,
        text_len: name.len() as u32,
        occ_first,
        occ_len: self.occ.len() as u32 - occ_first,
        encoding: encoding.code(),
      });
      set_bit(&mut self.coverage, self.paths.len() - 1, document.is_some());
    }
  }

  fn assemble_refs(&mut self) {
    let mut counts = vec![0u32; self.symbols.len()];
    for record in &self.occ {
      if let Some(count) = counts.get_mut(record.symbol as usize) {
        *count += 1;
      }
    }
    self.ref_offsets = Vec::with_capacity(counts.len() + 1);
    self.ref_offsets.push(0);
    let mut total = 0u32;
    for count in &counts {
      total += count;
      self.ref_offsets.push(total);
    }
    self.ref_entries = vec![0u32; total as usize];
    let mut cursor = self.ref_offsets.clone();
    for (id, record) in self.occ.iter().enumerate() {
      let Some(slot) = cursor.get_mut(record.symbol as usize) else {
        continue;
      };
      if let Some(entry) = self.ref_entries.get_mut(*slot as usize) {
        *entry = id as u32;
      }
      *slot += 1;
    }
  }
}

fn set_bit(bits: &mut Vec<u8>, index: usize, value: bool) {
  let byte = index / 8;
  if bits.len() <= byte {
    bits.resize(byte + 1, 0);
  }
  let mask = 0x80u8 >> (index % 8);
  if value {
    bits[byte] |= mask;
  } else {
    bits[byte] &= !mask;
  }
}

/// SCIP paths are repository-relative with `/` separators; keep them that way.
pub(crate) fn normalize(path: &str) -> Result<String> {
  let trimmed = path.trim_start_matches("./");
  let bad = trimmed.is_empty()
    || trimmed != trimmed.trim()
    || trimmed.starts_with('/')
    || trimmed.ends_with('/')
    || trimmed
      .split('/')
      .any(|part| part.is_empty() || part == "." || part == "..");
  if bad {
    return Err(Error::Path(path.to_string()));
  }
  Ok(trimmed.to_string())
}
