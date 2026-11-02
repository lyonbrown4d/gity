use gity_code_intel::{Kind, Language, SymbolFile};

const SOURCE: &str = r#"//! Crate level documentation.
use std::collections::HashMap;
use crate::store::{Index, Key};

/// A store of things.
///
/// Keeps entries in memory.
pub struct Store {
    entries: HashMap<String, usize>,
}

impl Store {
    /// Inserts one entry.
    pub fn insert(&mut self, key: String) {
        self.entries.insert(key, 0);
    }
}

pub enum Mode {
    Fast,
    Slow,
}

pub const LIMIT: usize = 8;

mod inner {
    pub fn helper() {}
}
"#;

fn store() -> SymbolFile {
  Language::Rust
    .parse("src/store.rs", SOURCE)
    .expect("rust source parses")
}

#[test]
fn extracts_declarations_with_kinds_and_scopes() {
  let file = store();
  let names = file
    .definitions
    .iter()
    .map(|definition| (definition.name.as_str(), definition.kind))
    .collect::<Vec<_>>();
  assert!(names.contains(&("Store", Kind::Struct)), "{names:?}");
  assert!(names.contains(&("insert", Kind::Function)), "{names:?}");
  assert!(names.contains(&("Mode", Kind::Enum)), "{names:?}");
  assert!(names.contains(&("LIMIT", Kind::Const)), "{names:?}");
  assert!(names.contains(&("helper", Kind::Function)), "{names:?}");
  assert!(names.contains(&("inner", Kind::Module)), "{names:?}");

  let insert = file
    .definitions
    .iter()
    .find(|definition| definition.name == "insert")
    .expect("method extracted");
  assert_eq!(insert.scope, "Store");
  assert_eq!(insert.kind, Kind::Function);
}

#[test]
fn captures_doc_comment_first_line() {
  let file = store();
  let store = file
    .definitions
    .iter()
    .find(|definition| definition.name == "Store")
    .expect("struct extracted");
  assert_eq!(store.doc, "A store of things.");
  let insert = file
    .definitions
    .iter()
    .find(|definition| definition.name == "insert")
    .expect("method extracted");
  assert_eq!(insert.doc, "Inserts one entry.");
}

#[test]
fn imports_are_split_into_module_and_names() {
  let file = store();
  let modules = file
    .imports
    .iter()
    .map(|import| import.module.as_str())
    .collect::<Vec<_>>();
  assert!(modules.contains(&"std::collections"), "{modules:?}");
  assert!(modules.contains(&"crate::store"), "{modules:?}");
  let store_import = file
    .imports
    .iter()
    .find(|import| import.module == "crate::store")
    .expect("braced import");
  assert_eq!(store_import.names, vec!["Index", "Key"]);
}

#[test]
fn definition_at_picks_the_innermost_declaration() {
  let file = store();
  let line = SOURCE
    .lines()
    .position(|line| line.contains("pub fn insert"))
    .expect("insert line") as u32
    + 1;
  let found = file
    .definition_at(line, 5)
    .expect("definition under cursor");
  assert_eq!(found.name, "insert");
  let outside = file.definition_at(1, 1);
  assert!(outside.is_none() || outside.is_some_and(|found| found.name == "Store"));
}
