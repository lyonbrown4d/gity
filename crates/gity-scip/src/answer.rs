//! Answers the precise layer returns, and how much it knows about a path.

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
  /// The symbol is defined outside this repository, so the position is the use
  /// site rather than a declaration to jump to.
  pub external: bool,
}

impl Location {
  pub fn span(&self) -> crate::model::Span {
    crate::model::Span::new(self.line, self.column, self.end_line, self.end_column)
  }
}

/// What the editor shows when hovering a symbol.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Hover {
  /// Raw SCIP symbol string, the key for [`crate::Shard::references`].
  pub symbol: String,
  /// Last descriptor of the symbol, for example `new`.
  pub display_name: String,
  /// Readable enclosing chain, for example `Config`.
  pub container: String,
  /// Package the symbol belongs to, empty when the symbol string has none.
  pub package: String,
  pub kind: crate::model::SymbolKind,
  pub documentation: String,
  pub signature: String,
  /// Readable enclosing declaration of the hovered occurrence, empty at file
  /// root. Unlike the raw symbol string this is safe to render directly.
  pub scope: String,
  /// The symbol is defined outside this repository.
  pub external: bool,
}
