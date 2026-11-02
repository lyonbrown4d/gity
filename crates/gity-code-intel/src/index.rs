//! Tantivy index over declarations, one directory per project.
//!
//! The index answers three questions the editor asks: what does this file
//! declare, which declaration covers a position, and where is a name declared.
use crate::symbol::{Kind, SymbolFile};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, STORED, STRING, Schema, Value};
use tantivy::{Index, TantivyDocument, Term};

pub struct Fields {
  pub path: Field,
  pub name: Field,
  pub kind: Field,
  pub scope: Field,
  pub line: Field,
  pub line_end: Field,
  pub column: Field,
  pub doc_text: Field,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolDoc {
  pub path: String,
  pub name: String,
  pub kind: String,
  pub scope: String,
  pub line: u64,
  pub line_end: u64,
  pub column: u64,
  pub doc: String,
}

pub fn schema() -> (Schema, Fields) {
  let mut builder = Schema::builder();
  let fields = Fields {
    path: builder.add_text_field("path", STRING | STORED),
    name: builder.add_text_field("name", STRING | STORED),
    kind: builder.add_text_field("kind", STRING | STORED),
    scope: builder.add_text_field("scope", STORED),
    line: builder.add_u64_field("line", STORED),
    line_end: builder.add_u64_field("line_end", STORED),
    column: builder.add_u64_field("column", STORED),
    doc_text: builder.add_text_field("doc_text", STORED),
  };
  (builder.build(), fields)
}

/// Rebuilds the index directory from scratch; callers own the commit lifecycle.
pub fn build(dir: &Path, files: &[SymbolFile]) -> Result<usize> {
  if dir.exists() {
    std::fs::remove_dir_all(dir).with_context(|| format!("clear {}", dir.display()))?;
  }
  std::fs::create_dir_all(dir).with_context(|| dir.display().to_string())?;
  let (schema, fields) = schema();
  let index = Index::create_in_dir(dir, schema).context("create symbol index")?;
  let mut writer = index.writer(50_000_000).context("open index writer")?;
  let mut written = 0usize;
  for file in files {
    for definition in &file.definitions {
      let mut document = TantivyDocument::default();
      document.add_text(fields.path, &file.path);
      document.add_text(fields.name, &definition.name);
      document.add_text(fields.kind, definition.kind.as_str());
      document.add_text(fields.scope, &definition.scope);
      document.add_u64(fields.line, definition.line as u64);
      document.add_u64(fields.line_end, definition.line_end as u64);
      document.add_u64(fields.column, definition.column as u64);
      document.add_text(fields.doc_text, &definition.doc);
      writer.add_document(document)?;
      written += 1;
    }
  }
  writer.commit().context("commit symbol index")?;
  Ok(written)
}

pub fn open(dir: &Path) -> Result<Option<(Index, Fields)>> {
  if !dir.join("meta.json").exists() {
    return Ok(None);
  }
  let index = Index::open_in_dir(dir).with_context(|| format!("open {}", dir.display()))?;
  let fields = Fields {
    path: index.schema().get_field("path")?,
    name: index.schema().get_field("name")?,
    kind: index.schema().get_field("kind")?,
    scope: index.schema().get_field("scope")?,
    line: index.schema().get_field("line")?,
    line_end: index.schema().get_field("line_end")?,
    column: index.schema().get_field("column")?,
    doc_text: index.schema().get_field("doc_text")?,
  };
  Ok(Some((index, fields)))
}

/// All declarations of one file, ordered by position.
pub fn symbols_in(index: &Index, fields: &Fields, path: &str) -> Result<Vec<SymbolDoc>> {
  let term = Term::from_field_text(fields.path, path);
  let query = tantivy::query::TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);
  let mut rows = collect(index, &query, 4096)?;
  rows.sort_by_key(|row| (row.line, row.column));
  Ok(rows)
}

/// Declarations whose name matches exactly; used for go-to-definition by name.
pub fn by_name(index: &Index, fields: &Fields, name: &str) -> Result<Vec<SymbolDoc>> {
  let term = Term::from_field_text(fields.name, name);
  let query = tantivy::query::TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);
  let mut rows = collect(index, &query, 512)?;
  rows.sort_by_key(|row| (row.path.clone(), row.line));
  Ok(rows)
}

/// Free-text search over declaration names, used by the outline panel.
pub fn search(index: &Index, fields: &Fields, text: &str, limit: usize) -> Result<Vec<SymbolDoc>> {
  let parser = QueryParser::for_index(index, vec![fields.name, fields.path]);
  let query = parser
    .parse_query(text)
    .with_context(|| format!("parse query {text}"))?;
  collect(index, &query, limit.clamp(1, 500))
}

fn collect(
  index: &Index,
  query: &dyn tantivy::query::Query,
  limit: usize,
) -> Result<Vec<SymbolDoc>> {
  let reader = index.reader().context("open symbol index reader")?;
  let searcher = reader.searcher();
  let hits = searcher
    .search(query, &TopDocs::with_limit(limit).order_by_score())
    .context("run symbol query")?;
  let mut rows = Vec::with_capacity(hits.len());
  for (_, address) in hits {
    let stored: TantivyDocument = searcher.doc(address)?;
    rows.push(SymbolDoc {
      path: text_of(&stored, index, "path"),
      name: text_of(&stored, index, "name"),
      kind: text_of(&stored, index, "kind"),
      scope: text_of(&stored, index, "scope"),
      line: number_of(&stored, index, "line"),
      line_end: number_of(&stored, index, "line_end"),
      column: number_of(&stored, index, "column"),
      doc: text_of(&stored, index, "doc_text"),
    });
  }
  Ok(rows)
}

fn text_of(doc: &TantivyDocument, index: &Index, name: &str) -> String {
  index
    .schema()
    .get_field(name)
    .ok()
    .and_then(|field| doc.get_first(field))
    .and_then(|value| value.as_str().map(str::to_owned))
    .unwrap_or_default()
}

fn number_of(doc: &TantivyDocument, index: &Index, name: &str) -> u64 {
  index
    .schema()
    .get_field(name)
    .ok()
    .and_then(|field| doc.get_first(field))
    .and_then(|value| value.as_u64())
    .unwrap_or(0)
}

/// Convenience for building the implicit scope chain of one file.
pub fn scope_of(definition: &SymbolDoc) -> String {
  if definition.scope.is_empty() {
    definition.name.clone()
  } else {
    format!("{}::{}", definition.scope, definition.name)
  }
}

/// Maps an indexed kind back to the enum for callers that need the literal.
pub fn kind_of(definition: &SymbolDoc) -> Option<Kind> {
  [
    Kind::Function,
    Kind::Method,
    Kind::Struct,
    Kind::Enum,
    Kind::Trait,
    Kind::Impl,
    Kind::Const,
    Kind::Static,
    Kind::TypeAlias,
    Kind::Module,
    Kind::Macro,
    Kind::Field,
    Kind::Variable,
  ]
  .into_iter()
  .find(|kind| kind.as_str() == definition.kind)
}
