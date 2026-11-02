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
/// different code units; the shard keeps it so callers can slice source text
/// with the right unit instead of guessing.
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
  pub text: String,
}

/// Everything a received `index.scip` contributes, independent of its storage.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Decoded {
  pub commit: String,
  pub indexer: String,
  pub indexer_version: String,
  pub documents: Vec<DecodedDocument>,
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

/// Result of a precise query, keeping "no answer" and "not indexed" apart.
///
/// `Uncovered` is the signal to fall back to the lexical layer. `Absent` is a
/// definite empty result inside a file the indexer did analyse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer<T> {
  Found(T),
  Absent,
  Uncovered,
}

impl<T> Answer<T> {
  pub fn is_found(&self) -> bool {
    matches!(self, Self::Found(_))
  }

  pub fn is_uncovered(&self) -> bool {
    matches!(self, Self::Uncovered)
  }

  pub fn into_option(self) -> Option<T> {
    match self {
      Self::Found(value) => Some(value),
      _ => None,
    }
  }

  pub fn map<U>(self, map: impl FnOnce(T) -> U) -> Answer<U> {
    match self {
      Self::Found(value) => Answer::Found(map(value)),
      Self::Absent => Answer::Absent,
      Self::Uncovered => Answer::Uncovered,
    }
  }
}

/// How much the shard knows about a path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
  /// The indexer analysed the file; empty answers are authoritative.
  Precise,
  /// The shard lists the file but holds no precise data for it.
  Listed,
  /// The shard never saw the path.
  Unknown,
}

impl Coverage {
  pub fn is_precise(self) -> bool {
    matches!(self, Self::Precise)
  }
}

/// A resolved source position in the repository.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Location {
  pub path: String,
  pub line: u32,
  pub column: u32,
  pub end_line: u32,
  pub end_column: u32,
}

impl Location {
  pub fn span(&self) -> Span {
    Span::new(self.line, self.column, self.end_line, self.end_column)
  }
}

/// What the editor shows when hovering a symbol.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Hover {
  pub symbol: String,
  pub kind: SymbolKind,
  pub documentation: String,
  pub signature: String,
  /// Enclosing definition of the hovered occurrence, empty at file root.
  pub scope: String,
}
