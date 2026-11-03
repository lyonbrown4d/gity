//! Exercises the scanner and store against real Rust sources in this repository.
use gity_code_intel::{Limits, Store, index::SymbolDoc};
use std::path::Path;

/// Resolves the workspace root from the test binary location.
fn workspace_root() -> &'static Path {
  Path::new(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .and_then(Path::parent)
    .expect("workspace root")
}

#[test]
fn scans_this_crates_sources_and_finds_known_symbols() {
  let root = workspace_root().join("crates/gity-ids/src");
  let files = gity_code_intel::scan::directory(&root, Limits::default()).expect("scan");
  assert!(
    !files.is_empty(),
    "no files scanned under {}",
    root.display()
  );
  assert!(files.iter().all(|file| file.language == "rust"));
  assert!(
    files.iter().any(|file| file.path.ends_with("snowflake.rs")),
    "paths: {:?}",
    files.iter().map(|file| &file.path).collect::<Vec<_>>()
  );

  let names = files
    .iter()
    .flat_map(|file| file.definitions.iter())
    .map(|definition| definition.name.as_str())
    .collect::<Vec<_>>();
  for expected in ["IdGenerator", "IdError", "install", "new_id"] {
    assert!(names.contains(&expected), "missing {expected} in {names:?}");
  }
}

#[test]
fn store_answers_lookups_over_real_files() {
  let temp = tempfile::tempdir().expect("temp dir");
  let store = Store::new(temp.path().join("intel")).expect("store");
  let root = workspace_root().join("crates/gity-ids/src");
  let status = store
    .refresh_directory(9, "test-commit", &root, Limits::default())
    .expect("index real sources");
  assert!(status.files >= 3, "{status:?}");
  assert!(status.definitions >= 10, "{status:?}");

  let file = if status.commit.is_empty() {
    ""
  } else {
    "snowflake.rs"
  };
  let symbols: Vec<SymbolDoc> = store.symbols(9, file).expect("symbols");
  // The path in the index is relative to the scanned root.
  let symbols = if symbols.is_empty() {
    let scanned = gity_code_intel::scan::directory(&root, Limits::default()).expect("scan");
    let path = scanned
      .iter()
      .find(|candidate| candidate.path.ends_with("snowflake.rs"))
      .map(|candidate| candidate.path.clone())
      .expect("snowflake path");
    store.symbols(9, &path).expect("symbols by scanned path")
  } else {
    symbols
  };
  assert!(
    symbols.iter().any(|symbol| symbol.name == "IdGenerator"),
    "symbols: {symbols:?}"
  );

  let doc = symbols
    .iter()
    .find(|symbol| symbol.name == "IdError")
    .and_then(|symbol| {
      store
        .hover_at(9, &symbol.path, symbol.line, symbol.column)
        .ok()
        .flatten()
    });
  assert!(doc.is_some(), "hover for IdError should resolve");
}
