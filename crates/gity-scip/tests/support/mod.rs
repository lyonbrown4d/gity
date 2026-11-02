//! Fixtures shared by the integration tests.
//!
//! Everything is a hand-built [`Decoded`] value, so the tests never need a real
//! indexer, a checkout or a network: exactly the intermediate shape `decode`
//! produces.
#![allow(dead_code)]

use gity_scip::{
  Decoded, DecodedDocument, DecodedOccurrence, DecodedSymbol, PositionEncoding, Span, SymbolKind,
  SymbolRole,
};

pub const COMMIT: &str = "0123456789abcdef";
pub const FOO: &str = "rust-analyzer cargo demo 0.1.0 src/Foo#";
pub const BAR: &str = "rust-analyzer cargo demo 0.1.0 src/bar().";
/// Path the runner reported but the indexer never analysed.
pub const SKIPPED: &str = "README.md";

pub fn occurrence(
  start_line: u32,
  start_column: u32,
  end_line: u32,
  end_column: u32,
  symbol: &str,
  roles: SymbolRole,
) -> DecodedOccurrence {
  DecodedOccurrence {
    span: Span::new(start_line, start_column, end_line, end_column),
    enclosing: None,
    symbol: symbol.to_string(),
    roles,
    syntax_kind: 0,
  }
}

pub fn symbol(symbol: &str, documentation: &str, signature: &str, kind: i32) -> DecodedSymbol {
  DecodedSymbol {
    symbol: symbol.to_string(),
    kind: SymbolKind::from_code(kind),
    documentation: documentation.to_string(),
    signature: signature.to_string(),
  }
}

pub fn document(
  path: &str,
  occurrences: Vec<DecodedOccurrence>,
  symbols: Vec<DecodedSymbol>,
) -> DecodedDocument {
  DecodedDocument {
    path: path.to_string(),
    language: "rust".to_string(),
    position_encoding: PositionEncoding::Utf8,
    occurrences,
    symbols,
    text: String::new(),
  }
}

/// `src/lib.rs` defines `Foo` and `bar`, `src/main.rs` uses `Foo`, and
/// `README.md` was skipped, which makes it a listed-but-uncovered path.
pub fn sample() -> Decoded {
  let mut decoded = Decoded::new(COMMIT);
  decoded.indexer = "rust-analyzer".to_string();
  decoded.indexer_version = "2026-10-01".to_string();
  decoded.uncovered_paths = vec![SKIPPED.to_string()];
  decoded.documents.push(document(
    "src/lib.rs",
    vec![
      occurrence(3, 8, 3, 11, FOO, SymbolRole::DEFINITION),
      // A multi-line definition so a reference inside it gets a scope.
      occurrence(10, 4, 13, 1, BAR, SymbolRole::DEFINITION),
      occurrence(12, 5, 12, 8, FOO, SymbolRole::READ),
      occurrence(20, 4, 20, 7, FOO, SymbolRole::READ),
      // Syntax-only occurrence: no symbol, never answers a lookup.
      occurrence(25, 1, 25, 3, "", SymbolRole::NONE),
    ],
    vec![
      symbol(FOO, "The Foo type.", "struct Foo", 7),
      symbol(BAR, "", "", 0),
    ],
  ));
  decoded.documents.push(document(
    "src/main.rs",
    vec![occurrence(5, 9, 5, 12, FOO, SymbolRole::READ)],
    Vec::new(),
  ));
  decoded
}

/// A shard directory inside a fresh temporary root.
pub fn root() -> (tempfile::TempDir, std::path::PathBuf) {
  let dir = tempfile::tempdir().expect("temp dir");
  let shard = dir.path().join("scip");
  (dir, shard)
}
