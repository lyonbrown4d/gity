//! Builds shard tables from a decoded index and reads them back.
//!
//! Ordering rules the query layer relies on:
//!
//! - paths are sorted by byte order, so path lookup is a binary search;
//! - each path owns a contiguous occurrence range, sorted by position, so a
//!   position lookup is another binary search;
//! - symbols are sorted by their SCIP string, so symbol lookup is a binary
//!   search, and occurrence IDs are assigned in path order so CSR entries for
//!   one symbol come out in repository order.

use crate::model::{Decoded, DecodedDocument, PositionEncoding, SymbolRole};
use crate::tables::{NONE, OccRecord, PathRecord, SymbolRecord};
use crate::{Error, Result};
use std::collections::{BTreeMap, BTreeSet};

/// How far back a nested occurrence searches for its enclosing definition.
const ENCLOSING_WINDOW: usize = 64;

/// Raw table bytes as they arrive from disk.
pub(crate) struct Raw {
  pub paths: Vec<u8>,
  pub occ: Vec<u8>,
  pub symbols: Vec<u8>,
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
  pub docs: Vec<u8>,
  pub ref_offsets: Vec<u32>,
  pub ref_entries: Vec<u32>,
  pub coverage: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
struct Agg {
  kind: i32,
  documentation: String,
  signature: String,
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
    let (symbols, symbol_pool, ids, docs) = assemble_symbols(precise.values().copied());
    let mut tables = Self {
      symbols,
      symbol_pool,
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
        fill_enclosing(&mut records);
        self.occ.extend_from_slice(&records);
      }
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

/// Collects every symbol string, sorts it, and assigns dense IDs.
fn assemble_symbols<'a>(
  documents: impl Iterator<Item = &'a DecodedDocument>,
) -> (Vec<SymbolRecord>, Vec<u8>, BTreeMap<&'a str, u32>, Vec<u8>) {
  let mut aggregated: BTreeMap<&'a str, Agg> = BTreeMap::new();
  for document in documents {
    for symbol in &document.symbols {
      if symbol.symbol.is_empty() {
        continue;
      }
      let agg = aggregated.entry(symbol.symbol.as_str()).or_default();
      if agg.kind == 0 {
        agg.kind = symbol.kind.code();
      }
      if agg.documentation.is_empty() {
        agg.documentation.clone_from(&symbol.documentation);
      }
      if agg.signature.is_empty() {
        agg.signature.clone_from(&symbol.signature);
      }
    }
    for occurrence in &document.occurrences {
      if !occurrence.symbol.is_empty() {
        aggregated.entry(occurrence.symbol.as_str()).or_default();
      }
    }
  }
  let mut pool = Vec::new();
  let mut docs = Vec::new();
  let mut records = Vec::with_capacity(aggregated.len());
  let mut ids = BTreeMap::new();
  for (index, (name, agg)) in aggregated.iter().enumerate() {
    ids.insert(*name, index as u32);
    let text_off = pool.len() as u32;
    pool.extend_from_slice(name.as_bytes());
    let doc_off = docs.len() as u32;
    docs.extend_from_slice(agg.documentation.as_bytes());
    let signature_off = docs.len() as u32;
    docs.extend_from_slice(agg.signature.as_bytes());
    records.push(SymbolRecord {
      text_off,
      text_len: name.len() as u32,
      kind: agg.kind as u32,
      doc_off,
      doc_len: agg.documentation.len() as u32,
      signature_off,
      signature_len: agg.signature.len() as u32,
    });
  }
  (records, pool, ids, docs)
}

/// Points each occurrence at the nearest enclosing definition occurrence.
fn fill_enclosing(records: &mut [OccRecord]) {
  for index in 0..records.len() {
    let start = index.saturating_sub(ENCLOSING_WINDOW);
    let target = records[index];
    let enclosing = (start..index).rev().find(|previous| {
      let candidate = &records[*previous];
      candidate.symbol != NONE
        && SymbolRole::from_bits(candidate.roles).is_definition()
        && candidate.covers(target.start_line, target.start_column)
    });
    records[index].enclosing = enclosing.map_or(NONE, |previous| records[previous].symbol);
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
