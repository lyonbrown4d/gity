//! Rust extractor: declarations, impl members and `use` edges via tree-sitter.
use crate::symbol::{Definition, Import, Kind, SymbolFile};
use anyhow::{Context, Result};
use tree_sitter::{Node, Parser};

/// Parses one Rust file into its declarations and imports.
pub fn parse(path: &str, source: &str) -> Result<SymbolFile> {
  let mut parser = Parser::new();
  parser
    .set_language(&tree_sitter_rust::LANGUAGE.into())
    .context("load tree-sitter Rust grammar")?;
  let tree = parser.parse(source, None).context("parse Rust source")?;
  let lines = source.lines().collect::<Vec<_>>();
  let mut file = SymbolFile::new(path, "rust");
  visit(tree.root_node(), source, &lines, "", &mut file);
  Ok(file)
}

fn visit(node: Node<'_>, source: &str, lines: &[&str], scope: &str, file: &mut SymbolFile) {
  let mut cursor = node.walk();
  for child in node.named_children(&mut cursor) {
    let kind = child.kind();
    if kind == "use_declaration" {
      if let Some(import) = import_of(child, source) {
        file.imports.push(import);
      }
      continue;
    }
    let Some(declared) = declaration_kind(kind) else {
      visit(child, source, lines, scope, file);
      continue;
    };
    let name = child
      .child_by_field_name("name")
      .and_then(|name| name.utf8_text(source.as_bytes()).ok())
      .unwrap_or_default();
    if !name.is_empty() {
      let (line, column) = position(child);
      file.definitions.push(Definition {
        name: name.to_owned(),
        kind: declared,
        scope: scope.to_owned(),
        line,
        line_end: child.end_position().row as u32 + 1,
        column,
        doc: doc(lines, line),
      });
    }
    // `impl` introduces a scope for its members; other blocks keep the parent scope.
    let inner = if kind == "impl_item" {
      impl_scope(child, source).unwrap_or_else(|| scope.to_owned())
    } else {
      scope.to_owned()
    };
    visit(child, source, lines, &inner, file);
  }
}

fn declaration_kind(kind: &str) -> Option<Kind> {
  Some(match kind {
    "function_item" => Kind::Function,
    "struct_item" => Kind::Struct,
    "enum_item" => Kind::Enum,
    "trait_item" => Kind::Trait,
    "impl_item" => Kind::Impl,
    "const_item" => Kind::Const,
    "static_item" => Kind::Static,
    "type_item" => Kind::TypeAlias,
    "mod_item" => Kind::Module,
    "macro_definition" => Kind::Macro,
    _ => return None,
  })
}

/// `impl Store for Client` keeps the self type as the scope of its members.
fn impl_scope(node: Node<'_>, source: &str) -> Option<String> {
  node
    .child_by_field_name("type")
    .and_then(|child| child.utf8_text(source.as_bytes()).ok())
    .map(|text| text.trim().to_owned())
    .filter(|text| !text.is_empty())
}

/// Turns a `use` declaration into the module path plus imported names.
fn import_of(node: Node<'_>, source: &str) -> Option<Import> {
  let text = node.utf8_text(source.as_bytes()).ok()?;
  let trimmed = text
    .trim_start_matches("pub ")
    .trim_start()
    .trim_start_matches("use ")
    .trim_end_matches(';')
    .trim();
  if trimmed.is_empty() {
    return None;
  }
  let (line, column) = position(node);
  let (module, names) = split_import(trimmed);
  Some(Import {
    line,
    column,
    module,
    names,
  })
}

/// Splits `crate::store::{Index, Key}` into `crate::store` and its names.
fn split_import(text: &str) -> (String, Vec<String>) {
  let text = text.trim();
  if let Some(open) = text.find('{') {
    let module = text[..open].trim_end_matches("::").to_owned();
    let inner = text[open + 1..].trim_end_matches('}').trim();
    let names = inner
      .split(',')
      .map(|name| name.trim().trim_start_matches("self as ").to_owned())
      .filter(|name| !name.is_empty() && name != "*")
      .collect();
    return (module, names);
  }
  match text.rsplit_once("::") {
    Some((module, name)) if name != "*" => (module.to_owned(), vec![name.to_owned()]),
    _ => (text.replace("::*", ""), Vec::new()),
  }
}

fn position(node: Node<'_>) -> (u32, u32) {
  let start = node.start_position();
  ((start.row + 1) as u32, (start.column + 1) as u32)
}

/// Collects the `///` block directly above a declaration, skipping attributes.
fn doc(lines: &[&str], line: u32) -> String {
  let mut collected = Vec::new();
  let mut cursor = line as usize; // 1-based number of the declared line
  while cursor > 1 {
    cursor -= 1; // moving to the line above
    let text = lines
      .get(cursor.saturating_sub(1))
      .map(|value| value.trim())
      .unwrap_or_default();
    if let Some(rest) = text
      .strip_prefix("///")
      .or_else(|| text.strip_prefix("//!"))
    {
      collected.push(rest.trim().to_owned());
      continue;
    }
    if text.starts_with("#[") || text.starts_with("#![") {
      continue;
    }
    break;
  }
  collected.reverse();
  collected
    .into_iter()
    .find(|text| !text.is_empty())
    .unwrap_or_default()
}
