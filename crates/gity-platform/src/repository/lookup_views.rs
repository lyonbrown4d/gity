use crate::openapi::code_schemas as spec;
use gity_code_intel::{Status, SymbolDoc};

pub(super) fn index_info(status: Option<&Status>) -> spec::CodeIndexInfo {
  spec::CodeIndexInfo {
    available: status.is_some(),
    commit: status
      .map(|status| status.commit.clone())
      .unwrap_or_default(),
    indexed_at: status
      .map(|status| status.indexed_at.clone())
      .unwrap_or_default(),
    truncated: status.map(|status| status.truncated).unwrap_or(false),
  }
}

pub(super) fn symbol(row: SymbolDoc) -> spec::CodeSymbol {
  spec::CodeSymbol {
    path: row.path,
    name: row.name,
    kind: row.kind,
    scope: row.scope,
    line: row.line,
    line_end: row.line_end,
    column: row.column,
    doc: row.doc,
  }
}
