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
//! [`Shard::definition_at`] and [`Shard::hover`] return [`Answer`], which keeps
//! two empty-looking outcomes apart:
//!
//! - [`Answer::Uncovered`]: the precise layer has nothing for this path. Fall
//!   back to the lexical layer.
//! - [`Answer::Absent`]: the indexer analysed this file and found no symbol at
//!   the cursor. This is a definite answer; do not fall back.
//!
//! Keys are always 1-based `(line, column)`, matching `gity-code-intel`'s
//! `Definition` and the editor API.

mod access;
mod assemble;
pub mod decode;
mod format;
pub mod model;
mod proto;
pub mod query;
pub mod shard;
mod tables;

pub use decode::{MAX_INDEX_BYTES, decode_index, decode_scip};
pub use model::{
  Answer, Coverage, Decoded, DecodedDocument, DecodedOccurrence, DecodedSymbol, Hover, Location,
  PositionEncoding, Span, SymbolKind, SymbolRole,
};
pub use shard::{Counts, CoverageSummary, Manifest, Shard};

/// Version of the on-disk shard format; bump when a table layout changes.
pub const FORMAT_VERSION: u32 = 1;

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
