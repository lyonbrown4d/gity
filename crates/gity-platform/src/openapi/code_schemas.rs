//! Response shapes for the code-intelligence lookups.
//!
//! Every answer carries `precision` and `index`, so a client can render
//! "approximate" without branching on which layer produced it. The lexical layer
//! is the only layer so far, so `precision` is always `approximate` and `mode` is
//! always `enclosing`; the fields exist now so that adding the precise layer later
//! does not change any response shape.
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct CodeSymbol {
  pub path: String,
  pub name: String,
  pub kind: String,
  #[serde(default)]
  pub scope: String,
  pub line: u64,
  pub line_end: u64,
  pub column: u64,
  #[serde(default)]
  pub doc: String,
}

#[derive(Serialize, ToSchema)]
pub struct CodeIndexInfo {
  pub available: bool,
  pub commit: String,
  pub indexed_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct CodeSymbols {
  pub path: String,
  pub items: Vec<CodeSymbol>,
  pub precision: String,
  pub index: CodeIndexInfo,
}

#[derive(Serialize, ToSchema)]
pub struct CodeDefinitions {
  pub definitions: Vec<CodeSymbol>,
  /// `enclosing` when the answer is the declaration surrounding the position.
  pub mode: String,
  pub precision: String,
  pub index: CodeIndexInfo,
}

#[derive(Serialize, ToSchema)]
pub struct CodeReference {
  pub path: String,
  pub name: String,
  pub line: u64,
  pub column: u64,
  pub scope: String,
}

#[derive(Serialize, ToSchema)]
pub struct CodeReferences {
  pub references: Vec<CodeReference>,
  pub total: u64,
  pub precision: String,
  pub index: CodeIndexInfo,
}

#[derive(Serialize, ToSchema)]
pub struct CodeHoverContent {
  pub contents_markdown: String,
  pub kind: String,
  /// `lexical` until a precise index answers; then `scip`.
  pub source: String,
}

#[derive(Serialize, ToSchema)]
pub struct CodeHover {
  pub hover: Option<CodeHoverContent>,
  pub precision: String,
  pub index: CodeIndexInfo,
}

#[derive(Serialize, ToSchema)]
pub struct CodeSearchResults {
  pub matches: Vec<CodeSymbol>,
  pub total: u64,
  pub precision: String,
  pub index: CodeIndexInfo,
}
