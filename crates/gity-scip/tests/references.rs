//! Reference expansion through the CSR: symbol string in, repository-ordered
//! occurrences out.

mod support;

use gity_scip::{Location, Shard};

fn open_sample() -> (tempfile::TempDir, Shard) {
  let (root, dir) = support::root();
  Shard::build(&dir, &support::sample()).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  (root, shard)
}

fn at(path: &str, line: u32, column: u32, end_line: u32, end_column: u32) -> Location {
  Location {
    path: path.to_string(),
    line,
    column,
    end_line,
    end_column,
    external: false,
  }
}

#[test]
fn references_expand_the_whole_csr_row() {
  let (_root, shard) = open_sample();
  assert_eq!(
    shard.references(support::FOO),
    vec![
      at("src/lib.rs", 3, 8, 3, 11),
      at("src/lib.rs", 12, 5, 12, 8),
      at("src/lib.rs", 20, 4, 20, 7),
      at("src/main.rs", 5, 9, 5, 12),
    ]
  );
}

#[test]
fn references_keep_definition_and_use_sites_together() {
  let (_root, shard) = open_sample();
  // `bar` is a multi-line definition, so its end position is on line 13.
  assert_eq!(
    shard.references(support::BAR),
    vec![at("src/lib.rs", 10, 4, 13, 1)]
  );
}

#[test]
fn references_of_an_unknown_symbol_are_empty() {
  let (_root, shard) = open_sample();
  assert!(
    shard
      .references("rust-analyzer cargo demo 0.1.0 src/nope().")
      .is_empty()
  );
  assert!(shard.references("").is_empty());
}

#[test]
fn references_survive_a_reopen() {
  let (_root, dir) = support::root();
  Shard::build(&dir, &support::sample()).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  drop(shard);
  let reopened = Shard::open(&dir).expect("open").expect("shard present");
  assert_eq!(reopened.references(support::FOO).len(), 4);
}

#[test]
fn references_do_not_leak_across_symbols() {
  let (_root, dir) = support::root();
  let mut decoded = support::sample();
  decoded.documents[0]
    .symbols
    .push(support::symbol("prefix of Foo", "", "", 0));
  Shard::build(&dir, &decoded).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  assert!(shard.references("prefix of Foo").is_empty());
  assert_eq!(shard.references(support::FOO).len(), 4);
}
