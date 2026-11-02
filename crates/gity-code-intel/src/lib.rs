//! Lexical code intelligence: declarations per file plus the archive transport.
//!
//! Precision is intentionally lexical: the extractor records what a file
//! declares and which modules it imports. Resolving a use site to a declaration
//! across files is the index layer's job, so this crate stays free of storage
//! and transport concerns.
pub mod archive;
pub mod index;
pub mod rust_lang;
pub mod scan;
pub mod store;
pub mod symbol;

pub use archive::{Archive, FORMAT_VERSION, decode, encode};
pub use index::SymbolDoc;
pub use scan::{Limits, ScanReport};
pub use store::{Status, Store};
pub use symbol::{Definition, Import, Kind, SymbolFile};

/// Languages the extractor currently understands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
  Rust,
}

impl Language {
  /// Guesses the language from a repository path.
  pub fn from_path(path: &str) -> Option<Self> {
    if path.ends_with(".rs") {
      return Some(Self::Rust);
    }
    None
  }
  pub fn parse(self, path: &str, source: &str) -> anyhow::Result<SymbolFile> {
    match self {
      Self::Rust => rust_lang::parse(path, source),
    }
  }
}
