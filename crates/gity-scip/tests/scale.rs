//! A large shard must open by mapping, not by reading everything into the heap.

mod support;

use gity_scip::{Answer, Coverage, Decoded, Shard, SymbolRole};

const PATHS: u32 = 2_000;
const PER_PATH: u32 = 100;

fn symbol_for(path: u32) -> String {
  format!("rust-analyzer cargo big 0.1.0 src/file_{path:04}.rs/item{path}().")
}

fn big() -> Decoded {
  let mut decoded = Decoded::new(support::COMMIT);
  decoded.indexer = "rust-analyzer".to_string();
  decoded.indexer_version = "2026-10-01".to_string();
  for path in 0..PATHS {
    let symbol = symbol_for(path);
    let mut occurrences = Vec::with_capacity(PER_PATH as usize);
    occurrences.push(support::occurrence(
      1,
      1,
      1,
      8,
      &symbol,
      SymbolRole::DEFINITION,
    ));
    for line in 2..=PER_PATH {
      occurrences.push(support::occurrence(
        line,
        5,
        line,
        12,
        &symbol,
        SymbolRole::READ,
      ));
    }
    decoded.documents.push(support::document(
      &format!("src/file_{path:04}.rs"),
      occurrences,
      vec![support::symbol(
        &symbol,
        "Item documentation.",
        "fn item()",
        26,
      )],
    ));
  }
  decoded
}

#[test]
fn a_large_shard_is_mapped_instead_of_copied() {
  let (_root, dir) = support::root();
  let manifest = Shard::build(&dir, &big()).expect("build");
  assert_eq!(manifest.counts.paths as u32, PATHS);
  assert_eq!(manifest.counts.occurrences as u32, PATHS * PER_PATH);

  let occ_bytes = std::fs::metadata(dir.join("occ.bin"))
    .expect("occ metadata")
    .len();
  assert!(
    occ_bytes > 5_000_000,
    "the fixture should produce a multi-megabyte occurrence table, got {occ_bytes}"
  );

  let shard = Shard::open(&dir).expect("open").expect("shard present");
  assert!(
    shard.resident_bytes() < 8 * 1024,
    "heap after opening a {occ_bytes} byte table: {}",
    shard.resident_bytes()
  );
  assert!(
    shard.mapped_bytes() as u64 >= occ_bytes,
    "mapped {} for a {occ_bytes} byte table",
    shard.mapped_bytes()
  );
}

#[test]
fn lookups_still_work_at_scale() {
  let (_root, dir) = support::root();
  Shard::build(&dir, &big()).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");

  let path = "src/file_1000.rs";
  assert_eq!(shard.coverage(path), Coverage::Precise);
  let answer = shard.definition_at(path, 60, 6);
  let Answer::Found(location) = answer else {
    panic!("expected a definition, got {answer:?}");
  };
  assert_eq!(location.path, path);
  assert_eq!((location.line, location.column), (1, 1));

  let symbol = symbol_for(1000);
  assert_eq!(shard.references(&symbol).len(), PER_PATH as usize);
  let Answer::Found(hover) = shard.hover(path, 60, 6) else {
    panic!("expected hover");
  };
  assert_eq!(hover.display_name, "item1000");
  assert_eq!(hover.documentation, "Item documentation.");
}

#[test]
fn path_lookup_stays_a_binary_search() {
  let (_root, dir) = support::root();
  Shard::build(&dir, &big()).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");

  // Both ends and a hole in the middle of the 2000 sorted paths.
  assert_eq!(shard.coverage("src/file_0000.rs"), Coverage::Precise);
  assert_eq!(shard.coverage("src/file_1999.rs"), Coverage::Precise);
  assert_eq!(shard.coverage("src/file_2000.rs"), Coverage::Unknown);
  assert_eq!(shard.coverage("src/file_0500.rs"), Coverage::Precise);
}

#[test]
fn opening_rejects_a_manifest_that_disagrees_with_the_bytes() {
  let (_root, dir) = support::root();
  Shard::build(&dir, &big()).expect("build");
  // Swap in a smaller shard's tables while keeping the big manifest.
  let (_other, replacement) = support::root();
  Shard::build(&replacement, &support::sample()).expect("build small");
  for name in ["occ.bin", "refs.bin"] {
    std::fs::rename(replacement.join(name), dir.join(name)).expect("rename");
  }
  assert!(Shard::open(&dir).expect("open").is_none());
  assert!(Shard::open_strict(&dir).is_err());
}
