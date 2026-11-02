//! Symbol data model shared by every language extractor.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
  Function,
  Method,
  Struct,
  Enum,
  Trait,
  Impl,
  Const,
  Static,
  TypeAlias,
  Module,
  Macro,
  Field,
  Variable,
}

impl Kind {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Function => "function",
      Self::Method => "method",
      Self::Struct => "struct",
      Self::Enum => "enum",
      Self::Trait => "trait",
      Self::Impl => "impl",
      Self::Const => "const",
      Self::Static => "static",
      Self::TypeAlias => "type",
      Self::Module => "module",
      Self::Macro => "macro",
      Self::Field => "field",
      Self::Variable => "variable",
    }
  }
}

/// One declared symbol. Lines and columns are 1-based, the same convention the
/// editor and the search API use.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
  pub name: String,
  pub kind: Kind,
  /// Enclosing path such as `crate::store::Index` or `impl Store`, empty at root.
  #[serde(default)]
  pub scope: String,
  pub line: u32,
  pub line_end: u32,
  pub column: u32,
  /// First paragraph of the doc comment, empty when undocumented.
  #[serde(default)]
  pub doc: String,
}

impl Definition {
  pub fn covers(&self, line: u32) -> bool {
    line >= self.line && line <= self.line_end
  }
}

/// A `use`/`import` statement, kept so the resolver can follow module edges.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Import {
  pub line: u32,
  pub column: u32,
  /// Module path as written in source, for example `crate::store` or `../lib`.
  pub module: String,
  /// Symbols imported from that module, empty for a glob or side-effect import.
  #[serde(default)]
  pub names: Vec<String>,
}

/// Everything one file contributes to the index.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolFile {
  pub path: String,
  pub language: String,
  #[serde(default)]
  pub definitions: Vec<Definition>,
  #[serde(default)]
  pub imports: Vec<Import>,
}

impl SymbolFile {
  pub fn new(path: impl Into<String>, language: impl Into<String>) -> Self {
    Self {
      path: path.into(),
      language: language.into(),
      definitions: Vec::new(),
      imports: Vec::new(),
    }
  }
  /// Innermost definition covering a position, preferring the latest start.
  pub fn definition_at(&self, line: u32, column: u32) -> Option<&Definition> {
    self
      .definitions
      .iter()
      .filter(|definition| definition.covers(line))
      .max_by_key(|definition| (definition.line, definition.column <= column))
  }
}
