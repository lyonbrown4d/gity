//! Queries over an open shard: definition, references, hover and coverage.
//!
//! Positions are 1-based, matching `gity-code-intel` and the editor API. Every
//! answer distinguishes "not covered" (degrade to the lexical layer) from
//! "covered, nothing here" (a definite empty result).

use crate::assemble::Tables;
use crate::model::{Answer, Coverage, Hover, Location, PositionEncoding, SymbolRole};
use crate::shard::Shard;
use crate::tables::NONE;

/// How far back a cursor searches for the occurrence it sits in. Occurrences
/// nest, so the window covers realistic expressions without scanning files.
const COVERING_WINDOW: usize = 256;

impl Shard {
  /// Whether the precise layer can answer for this path.
  pub fn coverage(&self, path: &str) -> Coverage {
    self.tables().coverage(path)
  }

  /// Resolves the symbol under the cursor to its definition, possibly in
  /// another file. `Uncovered` means the caller should use the lexical layer.
  pub fn definition_at(&self, path: &str, line: u32, column: u32) -> Answer<Location> {
    let tables = self.tables();
    let Some(path_id) = precise_path(tables, path) else {
      return Answer::Uncovered;
    };
    let Some(occurrence) = covering(tables, path_id, line, column) else {
      return Answer::Absent;
    };
    let Some(symbol) = tables.occurrence(occurrence).map(|record| record.symbol) else {
      return Answer::Absent;
    };
    tables
      .occurrences_of_symbol(symbol)
      .iter()
      .find(|candidate| {
        tables
          .occurrence(**candidate)
          .is_some_and(|record| SymbolRole::from_bits(record.roles).is_definition())
      })
      .and_then(|candidate| tables.location(*candidate))
      .map_or(Answer::Absent, Answer::Found)
  }

  /// Documentation and signature of the symbol under the cursor.
  pub fn hover(&self, path: &str, line: u32, column: u32) -> Answer<Hover> {
    let tables = self.tables();
    let Some(path_id) = precise_path(tables, path) else {
      return Answer::Uncovered;
    };
    let Some(occurrence) = covering(tables, path_id, line, column) else {
      return Answer::Absent;
    };
    let Some(record) = tables.occurrence(occurrence) else {
      return Answer::Absent;
    };
    let Some(symbol_id) = Some(record.symbol).filter(|id| *id != NONE) else {
      return Answer::Absent;
    };
    let Some(symbol) = tables.symbol(symbol_id) else {
      return Answer::Absent;
    };
    let scope = (record.enclosing != NONE)
      .then(|| tables.symbol(record.enclosing))
      .flatten()
      .unwrap_or_default();
    Answer::Found(Hover {
      symbol: symbol.to_string(),
      kind: tables.kind(symbol_id),
      documentation: tables.documentation(symbol_id).to_string(),
      signature: tables.signature(symbol_id).to_string(),
      scope: scope.to_string(),
    })
  }

  /// Every occurrence of a SCIP symbol string, in repository order.
  ///
  /// Symbol strings come from `hover`/`definition_at`; an unknown string yields
  /// an empty list rather than an error.
  pub fn references(&self, symbol: &str) -> Vec<Location> {
    let tables = self.tables();
    let Some(symbol_id) = tables.find_symbol(symbol) else {
      return Vec::new();
    };
    tables
      .occurrences_of_symbol(symbol_id)
      .iter()
      .filter_map(|occurrence| tables.location(*occurrence))
      .collect()
  }

  /// Column unit the indexer used for this path's ranges.
  pub fn position_encoding(&self, path: &str) -> Option<PositionEncoding> {
    let tables = self.tables();
    tables
      .find_path(path)
      .and_then(|path_id| tables.encoding(path_id))
  }
}

fn precise_path(tables: &Tables, path: &str) -> Option<u32> {
  let path_id = tables.find_path(path)?;
  tables.is_precise(path_id).then_some(path_id)
}

/// Best covering occurrence found so far, kept as "latest start, tightest end".
struct Covering {
  start: (u32, u32),
  end: (u32, u32),
  index: usize,
}

/// Global ID of the innermost occurrence whose span holds the cursor.
fn covering(tables: &Tables, path_id: u32, line: u32, column: u32) -> Option<u32> {
  let records = tables.occurrences(path_id);
  if records.is_empty() {
    return None;
  }
  let base = tables.occurrence_base(path_id);
  let limit =
    records.partition_point(|record| (record.start_line, record.start_column) <= (line, column));
  let mut best: Option<Covering> = None;
  let mut index = limit;
  let mut steps = 0;
  while index > 0 && steps < COVERING_WINDOW {
    index -= 1;
    steps += 1;
    let record = &records[index];
    if record.symbol == NONE || !record.covers(line, column) {
      continue;
    }
    let start = (record.start_line, record.start_column);
    let end = (record.end_line, record.end_column);
    let better = match &best {
      None => true,
      Some(found) => start > found.start || (start == found.start && end < found.end),
    };
    if better {
      best = Some(Covering { start, end, index });
    }
  }
  best.map(|found| base + found.index as u32)
}
