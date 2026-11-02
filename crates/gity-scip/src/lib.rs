//! SCIP precise code intelligence, layer 1: the local shard.
//!
//! The runner executes an indexer (`rust-analyzer scip`, `scip-typescript`, ...)
//! and stores the resulting `index.scip` zstd-compressed. The server decodes it
//! with [`decode_index`] and materialises a local shard with [`Shard::build`];
//! queries then run against fixed-layout tables only — no HTTP, no database, and
//! no dependency on `gity-code-intel`.
//!
//! Recommended placement mirrors the lexical snapshot's per-project directory,
//! so one root holds both layers:
//!
//! ```text
//! <code-intel root>/<project>/current/   lexical (tantivy) index
//! <code-intel root>/<project>/scip/      precise shard written by this crate
//! ```
//!
//! ## Answering a cursor request
//!
//! ```text
//! let column = shard.normalized_position(path, line, utf8_column)?;
//! match shard.definition_at(path, line, column) {
//!   Found(location) if !location.external => jump(location),
//!   Found(external) => show_hover(shard.hover(path, line, column)),
//!   Absent => definite: no symbol here,
//!   Uncovered => lexical fallback,
//! }
//! ```
//!
//! [`Answer::Uncovered`] is the only signal that should reach the lexical layer;
//! [`Answer::Absent`] is an authoritative empty result for a file the indexer
//! analysed. Keys are 1-based `(line, column)` in the document's own
//! [`PositionEncoding`], matching `gity-code-intel`'s `Definition`.
//!
//! ## Names and external symbols
//!
//! [`symbol_name`] turns a raw SCIP symbol string into display parts, and
//! [`Hover`] carries them. Symbols defined in dependencies are indexed from
//! `Index.external_symbols`: they answer hover, never coverage, never a
//! definition location, and never a phantom reference.

mod access;
mod answer;
mod assemble;
pub mod decode;
mod format;
mod manifest;
pub mod model;
mod naming;
mod proto;
pub mod query;
pub mod shard;
mod symbol_access;
mod symbols;
mod tables;
mod verify;

pub use answer::{Answer, Coverage, Hover, Location};
pub use decode::{MAX_INDEX_BYTES, decode_index, decode_scip};
pub use manifest::{Counts, CoverageSummary, Manifest};
pub use model::{
  Decoded, DecodedDocument, DecodedOccurrence, DecodedSymbol, PositionEncoding, Span, SymbolKind,
  SymbolRole,
};
pub use naming::{SymbolName, symbol_name};
pub use shard::Shard;

/// Version of the on-disk shard format; bump when a table layout changes.
///
/// - 1: paths, occ, symbols, docs, refs, coverage.
/// - 2: adds `external.bin` and `texts.bin`, and records the position encoding
///   in the manifest. Version 1 shards are rejected; rebuild them from the
///   stored `index.scip`, they hold no data of their own.
pub const FORMAT_VERSION: u32 = 2;

/// Artifact name the runner stores, zstd-compressed, per commit.
pub const INDEX_FILE: &str = "index.scip";

pub type Result<T> = std::result::Result<T, Error>;

/// Everything the precise layer can fail on: incoming artifacts and shard I/O.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("{path}: {source}")]
  Io {
    path: String,
    #[source]
    source: std::io::Error,
  },
  #[error("shard lock at {0} is held by another process")]
  Locked(String),
  #[error("shard format version {found} is not supported (expected {expected})")]
  Version { found: u32, expected: u32 },
  #[error("decompress index.scip: {0}")]
  Decompress(#[source] std::io::Error),
  #[error("index.scip exceeds the {limit} byte limit after decompression")]
  TooLarge { limit: usize },
  #[error("decode scip: {0}")]
  Proto(#[from] prost::DecodeError),
  #[error("manifest.json: {0}")]
  Manifest(#[from] serde_json::Error),
  #[error("shard table {table} is truncated or was not written by gity-scip")]
  Corrupt { table: &'static str },
  #[error("shard tables disagree with each other; rebuild the shard")]
  Inconsistent,
  #[error("{0:?} is not a repository-relative path")]
  Path(String),
  #[error("the index lists document {0:?} more than once")]
  DuplicatePath(String),
}

#[cfg(test)]
mod tests;
