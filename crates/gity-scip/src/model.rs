//! Platform-shaped data model for the precise layer.
//!
//! Positions are 1-based everywhere in this crate, matching `gity-code-intel`
//! and the editor API. SCIP's 0-based offsets only exist inside the decoder,
//! which converts them while building the intermediate structures below.

use serde::{Deserialize, Serialize};

/// A half-open source span, 1-based, the same orientation as `Definition`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Span {
  pub start_line: u32,
  pub start_column: u32,
  pub end_line: u32,
  pub end_column: u32,
}

impl Span {
  pub fn new(start_line: u32, start_column: u32, end_line: u32, end_column: u32) -> Self {
    Self {
      start_line,
      start_column,
      end_line,
      end_column,
    }
  }

  /// Whether a cursor placed at `(line, column)` sits inside the span.
  pub fn covers(&self, line: u32, column: u32) -> bool {
    (line, column) >= (self.start_line, self.start_column)
      && (line, column) < (self.end_line, self.end_column)
  }
}

/// How a range's character offsets count columns.
///
/// SCIP records this per document because indexers in different languages count
/// different code units: Go and Rust indexers count UTF-8 bytes, JVM and
/// JavaScript indexers count UTF-16 code units, Python indexers count `char`s.
/// The shard stores the raw unit and keeps the encoding next to it, so callers
/// can convert instead of silently slicing text at the wrong offset.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionEncoding {
  #[default]
  Unspecified,
  Utf8,
  Utf16,
  Utf32,
}

impl PositionEncoding {
  pub fn code(self) -> u32 {
    match self {
      Self::Unspecified => 0,
      Self::Utf8 => 1,
      Self::Utf16 => 2,
      Self::Utf32 => 3,
    }
  }

  pub fn from_code(code: i32) -> Self {
    match code {
      1 => Self::Utf8,
      2 => Self::Utf16,
      3 => Self::Utf32,
      _ => Self::Unspecified,
    }
  }

  pub fn as_str(self) -> &'static str {
    match self {
      Self::Unspecified => "unspecified",
      Self::Utf8 => "utf8",
      Self::Utf16 => "utf16",
      Self::Utf32 => "utf32",
    }
  }

  /// Converts a 1-based UTF-8 byte column in `line` to this encoding's unit.
  ///
  /// `utf8_column` is what slicing the repository source yields; the result
  /// compares directly against occurrence spans in the shard. Columns past the
  /// end of the line clamp to the line end, and a column inside a multi-byte
  /// character rounds down to its start.
  pub fn column(self, line: &str, utf8_column: u32) -> u32 {
    let prefix = prefix_of(line, utf8_column);
    let units = match self {
      Self::Utf8 | Self::Unspecified => prefix.len(),
      Self::Utf16 => prefix.encode_utf16().count(),
      Self::Utf32 => prefix.chars().count(),
    };
    u32::try_from(units).unwrap_or(u32::MAX).saturating_add(1)
  }
}

/// The part of `line` before a 1-based UTF-8 byte column, on a char boundary.
fn prefix_of(line: &str, utf8_column: u32) -> &str {
  let mut end = usize::try_from(utf8_column.saturating_sub(1))
    .unwrap_or(usize::MAX)
    .min(line.len());
  while !line.is_char_boundary(end) {
    end -= 1;
  }
  &line[..end]
}

/// What an occurrence does with its symbol, using SCIP's `SymbolRole` bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SymbolRole(u32);

impl SymbolRole {
  pub const NONE: Self = Self(0);
  pub const DEFINITION: Self = Self(0x1);
  pub const IMPORT: Self = Self(0x2);
  pub const WRITE: Self = Self(0x4);
  pub const READ: Self = Self(0x8);
  pub const GENERATED: Self = Self(0x10);
  pub const TEST: Self = Self(0x20);
  pub const FORWARD: Self = Self(0x40);

  pub fn from_bits(bits: u32) -> Self {
    Self(bits)
  }

  pub fn bits(self) -> u32 {
    self.0
  }

  pub fn contains(self, other: Self) -> bool {
    self.0 & other.0 == other.0
  }

  pub fn is_definition(self) -> bool {
    self.contains(Self::DEFINITION)
  }
}

/// SCIP symbol kind, kept as the wire code so upgrades do not need a shard
/// rebuild. Use [`SymbolKind::name`] for display.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolKind(pub i32);

impl SymbolKind {
  pub fn code(self) -> i32 {
    self.0
  }

  pub fn from_code(code: i32) -> Self {
    Self(code)
  }

  /// Canonical SCIP spelling such as `Method`; `Unknown` for fresh codes.
  pub fn name(self) -> &'static str {
    crate::proto::symbol_information::Kind::try_from(self.0)
      .map(|kind| kind.as_str_name())
      .unwrap_or("Unknown")
  }
}

/// One occurrence as decoded from `index.scip`, already 1-based.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DecodedOccurrence {
  pub span: Span,
  pub enclosing: Option<Span>,
  /// SCIP symbol string; empty for pure syntax-highlighting occurrences.
  pub symbol: String,
  pub roles: SymbolRole,
  pub syntax_kind: i32,
}

/// One symbol record as decoded from `index.scip`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DecodedSymbol {
  pub symbol: String,
  pub kind: SymbolKind,
  pub documentation: String,
  pub signature: String,
}

/// One document as decoded from `index.scip`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DecodedDocument {
  pub path: String,
  pub language: String,
  pub position_encoding: PositionEncoding,
  pub occurrences: Vec<DecodedOccurrence>,
  pub symbols: Vec<DecodedSymbol>,
  /// Source text when the indexer included it. Most indexers omit it; keeping
  /// it lets column conversion stay exact for the ones that do.
  pub text: String,
}

/// Everything a received `index.scip` contributes, independent of its storage.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Decoded {
  pub commit: String,
  pub indexer: String,
  pub indexer_version: String,
  pub documents: Vec<DecodedDocument>,
  /// Symbols defined outside this repository (`Index.external_symbols`), kept
  /// for hover only: they have no path, no occurrence and no coverage bit.
  pub external: Vec<DecodedSymbol>,
  /// Paths the runner knows but the indexer did not analyse. Keeping them turns
  /// "unknown file" into "known file, no precise data" for the caller.
  pub uncovered_paths: Vec<String>,
}

impl Decoded {
  pub fn new(commit: impl Into<String>) -> Self {
    Self {
      commit: commit.into(),
      ..Self::default()
    }
  }

  /// Adds a document and returns it for further edits.
  pub fn push_document(&mut self, document: DecodedDocument) -> &mut DecodedDocument {
    self.documents.push(document);
    let last = self.documents.len() - 1;
    &mut self.documents[last]
  }
}
