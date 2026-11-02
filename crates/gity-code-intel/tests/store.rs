use gity_code_intel::{Archive, Language, Store, SymbolDoc};

const SOURCE: &str = r#"/// Entry point.
pub fn run() {}

pub struct Config {
    pub retries: usize,
}

impl Config {
    pub fn new() -> Self {
        Self { retries: 3 }
    }
}
"#;

fn store() -> (tempfile::TempDir, Store) {
  let temp = tempfile::tempdir().expect("temp dir");
  let store = Store::new(temp.path().join("intel")).expect("store");
  (temp, store)
}

fn archive() -> Archive {
  let file = Language::Rust
    .parse("src/app.rs", SOURCE)
    .expect("parse source");
  Archive::new("abc123", vec![file])
}

#[test]
fn refresh_builds_an_index_and_reports_status() {
  let (_temp, store) = store();
  let status = store.refresh(7, &archive()).expect("refresh");
  assert_eq!(status.commit, "abc123");
  assert_eq!(status.files, 1);
  // `impl` blocks are scopes rather than declarations, so three symbols
  // (run, Config, new) are expected here.
  assert!(status.definitions >= 3, "{status:?}");
  assert!(status.archive_bytes > 0);

  let read = store.status(7).expect("status").expect("indexed");
  assert_eq!(read, status);
}

#[test]
fn symbols_are_ordered_by_position() {
  let (_temp, store) = store();
  store.refresh(7, &archive()).expect("refresh");
  let symbols: Vec<SymbolDoc> = store.symbols(7, "src/app.rs").expect("symbols");
  let names: Vec<&str> = symbols.iter().map(|symbol| symbol.name.as_str()).collect();
  assert_eq!(names.first(), Some(&"run"), "{names:?}");
  assert!(symbols.windows(2).all(|pair| pair[0].line <= pair[1].line));
}

#[test]
fn definition_at_returns_the_enclosing_symbol_with_doc() {
  let (_temp, store) = store();
  store.refresh(7, &archive()).expect("refresh");
  let run_line = 2;
  let found = store
    .definition_at(7, "src/app.rs", run_line, 4)
    .expect("lookup")
    .expect("definition");
  assert_eq!(found.name, "run");

  let no_index = store.definition_at(8, "src/app.rs", run_line, 4);
  assert!(no_index.expect("missing project").is_none());

  let hover = store
    .hover_at(7, "src/app.rs", run_line, 4)
    .expect("hover")
    .expect("hover doc");
  assert_eq!(hover.doc, "Entry point.");
}

#[test]
fn references_find_declarations_by_exact_name() {
  let (_temp, store) = store();
  store.refresh(7, &archive()).expect("refresh");
  let hits = store.references(7, "new").expect("references");
  assert_eq!(hits.len(), 1);
  assert_eq!(hits[0].scope, "Config");
}

#[test]
fn refresh_replaces_the_previous_snapshot() {
  let (_temp, store) = store();
  store.refresh(7, &archive()).expect("first");
  let empty = Archive::new("def456", Vec::new());
  let status = store.refresh(7, &empty).expect("second");
  assert_eq!(status.commit, "def456");
  assert_eq!(store.symbols(7, "src/app.rs").expect("symbols").len(), 0);
  store.forget(7).expect("forget");
  assert!(store.status(7).expect("status").is_none());
}
