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
/// A dependency symbol: documented in the index, defined outside the repository.
pub const DEP: &str = "scip-rust cargo serde 1.0.0 serde/de/Deserialize#deserialize().";
/// A dependency symbol nobody in this repository uses.
pub const DEP_UNUSED: &str = "scip-rust cargo serde 1.0.0 serde/de/Serialize#serialize().";
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

/// One analysed file that uses two dependency symbols, one of which the indexer
/// documented and one nobody uses.
pub fn external_sample() -> Decoded {
  let mut decoded = Decoded::new(COMMIT);
  decoded.indexer = "scip-rust".to_string();
  decoded.indexer_version = "0.4.0".to_string();
  decoded.documents.push(document(
    "src/use_dep.rs",
    vec![occurrence(4, 5, 4, 16, DEP, SymbolRole::READ)],
    Vec::new(),
  ));
  decoded.external = vec![
    symbol(
      DEP,
      "Deserialize a value from the wire.",
      "fn deserialize()",
      26,
    ),
    symbol(DEP_UNUSED, "Unused documentation.", "", 26),
  ];
  decoded
}

/// A UTF-16 document whose first line mixes an emoji, CJK text and ASCII, plus
/// one UTF-16 document without text: the two cases column conversion has to
/// distinguish.
pub fn utf16_sample() -> Decoded {
  let mut decoded = Decoded::new(COMMIT);
  decoded.indexer = "scip-python".to_string();
  decoded.indexer_version = "0.3.0".to_string();
  let mut source = document(
    "src/emoji.rs",
    // `Foo` starts at UTF-16 column 11 (1-based) on line 1.
    vec![occurrence(1, 11, 1, 14, FOO, SymbolRole::DEFINITION)],
    vec![symbol(FOO, "The Foo type.", "struct Foo", 7)],
  );
  source.position_encoding = PositionEncoding::Utf16;
  source.text = UTF16_TEXT.to_string();
  decoded.documents.push(source);
  let mut untracked = document("src/no_text.rs", Vec::new(), Vec::new());
  untracked.position_encoding = PositionEncoding::Utf16;
  decoded.documents.push(untracked);
  decoded
}

/// `let 🚀名 = Foo;` — 14 UTF-8 bytes of prefix, 10 UTF-16 units, 9 chars.
pub const UTF16_TEXT: &str = "let 🚀名 = Foo;\n// tail\n";

/// A shard directory inside a fresh temporary root.
pub fn root() -> (tempfile::TempDir, std::path::PathBuf) {
  let dir = tempfile::tempdir().expect("temp dir");
  let shard = dir.path().join("scip");
  (dir, shard)
}
