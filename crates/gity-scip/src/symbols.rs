//! Symbol aggregation: dense IDs, documentation merging, enclosing scopes.
//!
//! Repository symbols and external symbols share the record layout and the
//! documentation blob, but never the IDs: only repository symbols get an ID that
//! occurrences and the CSR can reference.

use crate::model::{DecodedDocument, DecodedSymbol, SymbolRole};
use crate::tables::{NONE, OccRecord, SymbolRecord};
use std::collections::BTreeMap;

/// How far back a nested occurrence searches for its enclosing definition.
const ENCLOSING_WINDOW: usize = 64;

/// Documentation and kind accumulated for one symbol string.
#[derive(Clone, Debug, Default)]
pub(crate) struct Aggregate {
  kind: i32,
  documentation: String,
  signature: String,
}

impl Aggregate {
  /// First non-empty kind, documentation and signature win; later documents
  /// usually repeat the same text.
  fn merge(&mut self, symbol: &DecodedSymbol) {
    if self.kind == 0 {
      self.kind = symbol.kind.code();
    }
    if self.documentation.is_empty() {
      self.documentation.clone_from(&symbol.documentation);
    }
    if self.signature.is_empty() {
      self.signature.clone_from(&symbol.signature);
    }
  }
}

/// Symbols declared in the analysed documents plus every symbol an occurrence
/// names, so a use site always has a record even without documentation.
pub(crate) fn document_aggregates<'a>(
  documents: impl Iterator<Item = &'a DecodedDocument>,
) -> BTreeMap<&'a str, Aggregate> {
  let mut aggregated: BTreeMap<&'a str, Aggregate> = BTreeMap::new();
  for document in documents {
    for symbol in &document.symbols {
      if !symbol.symbol.is_empty() {
        aggregated
          .entry(symbol.symbol.as_str())
          .or_default()
          .merge(symbol);
      }
    }
    for occurrence in &document.occurrences {
      if !occurrence.symbol.is_empty() {
        aggregated.entry(occurrence.symbol.as_str()).or_default();
      }
    }
  }
  aggregated
}

/// Symbols the indexer says are defined outside this repository.
pub(crate) fn external_aggregates(symbols: &[DecodedSymbol]) -> BTreeMap<&str, Aggregate> {
  let mut aggregated: BTreeMap<&str, Aggregate> = BTreeMap::new();
  for symbol in symbols {
    if !symbol.symbol.is_empty() {
      aggregated
        .entry(symbol.symbol.as_str())
        .or_default()
        .merge(symbol);
    }
  }
  aggregated
}

/// Writes records in symbol order and returns their strings' dense IDs.
pub(crate) fn assign<'a>(
  aggregated: BTreeMap<&'a str, Aggregate>,
  pool: &mut Vec<u8>,
  docs: &mut Vec<u8>,
) -> (Vec<SymbolRecord>, BTreeMap<&'a str, u32>) {
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
  (records, ids)
}

/// Points each occurrence at the nearest enclosing definition occurrence.
pub(crate) fn fill_enclosing(records: &mut [OccRecord]) {
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
