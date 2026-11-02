//! Queries over an open shard: definition, references, hover and coverage.
//!
//! Positions are 1-based, matching `gity-code-intel` and the editor API, and are
//! expressed in the document's own [`PositionEncoding`] — use
//! [`Shard::normalized_position`] to convert a UTF-8 column first. Every answer
//! distinguishes "not covered" (degrade to the lexical layer) from "covered,
//! nothing here" (a definite empty result).

use crate::answer::{Answer, Coverage, Hover, Location};
use crate::assemble::Tables;
use crate::model::{PositionEncoding, SymbolRole};
use crate::naming::symbol_name;
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
  ///
  /// A symbol defined in a dependency has no repository position, so the answer
  /// is `Found` with `external: true` and the position of the use site: show its
  /// hover documentation instead of navigating.
  pub fn definition_at(&self, path: &str, line: u32, column: u32) -> Answer<Location> {
    let tables = self.tables();
    let Some(path_id) = precise_path(tables, path) else {
      return Answer::Uncovered;
    };
    let Some(occurrence) = covering(tables, path_id, line, column) else {
      return Answer::Absent;
    };
    let Some(record) = tables.occurrence(occurrence).copied() else {
      return Answer::Absent;
    };
    if let Some(location) = definition_location(tables, record.symbol) {
      return Answer::Found(location);
    }
    // The use site created a repository symbol record for the occurrence, so
    // ask the external table directly: it is what the indexer reported as
    // defined elsewhere.
    let external = tables
      .symbol(record.symbol)
      .is_some_and(|raw| tables.find_external(raw).is_some());
    match tables.location(occurrence) {
      Some(mut location) if external => {
        location.external = true;
        Answer::Found(location)
      }
      _ => Answer::Absent,
    }
  }

  /// Documentation and signature of the symbol under the cursor.
  ///
  /// When the indexer sent documentation for a dependency symbol it fills the
  /// gaps left by this repository's own record, and `external` says whether the
  /// symbol is defined outside the repository.
  pub fn hover(&self, path: &str, line: u32, column: u32) -> Answer<Hover> {
    let tables = self.tables();
    let Some(path_id) = precise_path(tables, path) else {
      return Answer::Uncovered;
    };
    let Some(occurrence) = covering(tables, path_id, line, column) else {
      return Answer::Absent;
    };
    let Some(record) = tables.occurrence(occurrence).copied() else {
      return Answer::Absent;
    };
    let Some(raw) = tables.symbol(record.symbol) else {
      return Answer::Absent;
    };
    let name = symbol_name(raw);
    let mut documentation = tables.documentation(record.symbol).to_string();
    let mut signature = tables.signature(record.symbol).to_string();
    let mut kind = tables.kind(record.symbol);
    let external_id = tables.find_external(raw);
    if let Some(external_id) = external_id {
      if documentation.is_empty() {
        documentation = tables.external_documentation(external_id).to_string();
      }
      if signature.is_empty() {
        signature = tables.external_signature(external_id).to_string();
      }
      if kind.code() == 0 {
        kind = tables.external_kind(external_id);
      }
    }
    // A symbol the repository also declares is ours, even when the indexer
    // listed it as external for other packages.
    let external = external_id.is_some() && definition_location(tables, record.symbol).is_none();
    let scope = tables
      .symbol(record.enclosing)
      .filter(|_| record.enclosing != NONE)
      .map(|enclosing| symbol_name(enclosing).path())
      .unwrap_or_default();
    Answer::Found(Hover {
      symbol: raw.to_string(),
      display_name: name.display_name,
      container: name.container,
      package: name.package,
      kind,
      documentation,
      signature,
      scope,
      external,
    })
  }

  /// Every repository occurrence of a SCIP symbol string, in file order.
  ///
  /// Only occurrences in this repository are counted: a symbol defined in a
  /// dependency contributes its use sites (if any) and nothing else, so callers
  /// never see a reference they cannot open. Symbol strings come from `hover`;
  /// an unknown string yields an empty list rather than an error.
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

  /// Converts a 1-based UTF-8 byte column from the repository source into the
  /// column unit this path's occurrences use, ready for [`Shard::definition_at`].
  ///
  /// Exact when the indexer shipped the document text. Returns the column
  /// unchanged for UTF-8 documents, and `None` for a UTF-16/UTF-32 document that
  /// shipped no text, where the caller must count units from its own copy of the
  /// line.
  pub fn normalized_position(&self, path: &str, line: u32, utf8_column: u32) -> Option<u32> {
    let tables = self.tables();
    let path_id = tables.find_path(path)?;
    let encoding = tables.encoding(path_id)?;
    if matches!(
      encoding,
      PositionEncoding::Utf8 | PositionEncoding::Unspecified
    ) {
      return Some(utf8_column.max(1));
    }
    let line_text = tables
      .text(path_id)?
      .split('\n')
      .nth(usize::try_from(line).ok()?.checked_sub(1)?)?;
    Some(encoding.column(line_text, utf8_column))
  }

  /// Source text stored with the document, when the indexer shipped it.
  pub fn source_text(&self, path: &str) -> Option<&str> {
    let tables = self.tables();
    tables
      .find_path(path)
      .and_then(|path_id| tables.text(path_id))
  }
}

fn precise_path(tables: &Tables, path: &str) -> Option<u32> {
  let path_id = tables.find_path(path)?;
  tables.is_precise(path_id).then_some(path_id)
}

fn definition_location(tables: &Tables, symbol_id: u32) -> Option<Location> {
  tables
    .occurrences_of_symbol(symbol_id)
    .iter()
    .find(|candidate| {
      tables
        .occurrence(**candidate)
        .is_some_and(|record| SymbolRole::from_bits(record.roles).is_definition())
    })
    .and_then(|candidate| tables.location(*candidate))
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
