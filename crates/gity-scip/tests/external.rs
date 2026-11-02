//! External symbols: hover answers for dependencies, and nothing else changes.

mod support;

use gity_scip::{Answer, Coverage, Shard};

fn open_external() -> (tempfile::TempDir, Shard) {
  let (root, dir) = support::root();
  Shard::build(&dir, &support::external_sample()).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  (root, shard)
}

#[test]
fn hover_documents_a_symbol_from_a_dependency() {
  let (_root, shard) = open_external();
  let Answer::Found(hover) = shard.hover("src/use_dep.rs", 4, 6) else {
    panic!("expected hover on the external symbol");
  };
  assert!(hover.external);
  assert_eq!(hover.symbol, support::DEP);
  assert_eq!(hover.display_name, "deserialize");
  assert_eq!(hover.container, "Deserialize");
  assert_eq!(hover.package, "serde");
  assert_eq!(hover.documentation, "Deserialize a value from the wire.");
  assert_eq!(hover.signature, "fn deserialize()");
  assert_eq!(hover.kind.name(), "Method");
}

#[test]
fn definition_at_marks_an_external_symbol_instead_of_jumping() {
  let (_root, shard) = open_external();
  let Answer::Found(location) = shard.definition_at("src/use_dep.rs", 4, 6) else {
    panic!("expected a found answer for the external symbol");
  };
  assert!(
    location.external,
    "caller must not navigate to a dependency"
  );
  // The position is the use site, since no declaration exists in the checkout.
  assert_eq!(location.path, "src/use_dep.rs");
  assert_eq!((location.line, location.column), (4, 5));
}

#[test]
fn external_symbols_never_create_references() {
  let (_root, shard) = open_external();
  let references = shard.references(support::DEP);
  assert_eq!(references.len(), 1, "only the in-repository use site");
  assert_eq!(references[0].path, "src/use_dep.rs");
  // A documented dependency symbol nobody uses stays invisible to references.
  assert!(shard.references(support::DEP_UNUSED).is_empty());
}

#[test]
fn external_symbols_do_not_change_coverage() {
  let (_root, dir) = support::root();
  let manifest = Shard::build(&dir, &support::external_sample()).expect("build");
  assert_eq!(manifest.coverage.precise_paths, 1);
  assert_eq!(manifest.coverage.listed_paths, 0);
  assert_eq!(manifest.counts.external_symbols, 2);
  assert_eq!(
    manifest.counts.symbols, 1,
    "only the use-site occurrence needs a repository symbol"
  );
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  assert_eq!(shard.coverage("src/use_dep.rs"), Coverage::Precise);
}

#[test]
fn a_repository_definition_outranks_an_external_entry() {
  let (_root, dir) = support::root();
  let mut decoded = support::external_sample();
  // The indexer listed the symbol as external, but this repository defines it
  // too (for example a re-export checked into the tree).
  decoded.documents[0].symbols.push(support::symbol(
    support::DEP,
    "Local re-export documentation.",
    "",
    26,
  ));
  decoded.documents[0].occurrences.push(support::occurrence(
    6,
    1,
    6,
    10,
    support::DEP,
    gity_scip::SymbolRole::DEFINITION,
  ));
  Shard::build(&dir, &decoded).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  let Answer::Found(location) = shard.definition_at("src/use_dep.rs", 4, 6) else {
    panic!("expected the local definition");
  };
  assert!(!location.external);
  assert_eq!((location.line, location.column), (6, 1));
  let Answer::Found(hover) = shard.hover("src/use_dep.rs", 4, 6) else {
    panic!("expected hover");
  };
  assert!(!hover.external);
  assert_eq!(hover.documentation, "Local re-export documentation.");
}
