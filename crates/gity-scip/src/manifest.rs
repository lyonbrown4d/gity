//! The manifest that describes one shard: what produced it and what is inside.
//!
//! It is written last and is the commit point of a build, the same role
//! `status.json` plays for the lexical snapshot.

use serde::{Deserialize, Serialize};

/// What a shard holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
  pub format_version: u32,
  pub commit: String,
  pub indexer: String,
  pub indexer_version: String,
  /// RFC 3339, same spelling as the lexical status document.
  pub indexed_at: String,
  /// Column unit the analysed documents use: `utf8`, `utf16`, `utf32`,
  /// `unspecified`, or `mixed` when documents disagree. Occurrence columns are
  /// in this unit; use [`crate::Shard::normalized_position`] to convert.
  pub position_encoding: String,
  pub counts: Counts,
  pub coverage: CoverageSummary,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
  pub paths: usize,
  pub occurrences: usize,
  pub symbols: usize,
  /// Symbols defined outside the repository, kept for hover only.
  pub external_symbols: usize,
  /// CSR entries: occurrences that carry a symbol.
  pub references: usize,
  pub doc_bytes: usize,
  /// Source text the indexer shipped; usually 0.
  pub text_bytes: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageSummary {
  pub total_paths: usize,
  pub precise_paths: usize,
  /// Paths the runner reported but the indexer did not analyse.
  pub listed_paths: usize,
}
