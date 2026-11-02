//! What happens to an open shard whose files change underneath it.

mod support;

use gity_scip::{Answer, Coverage, Error, Shard};

const FILES: [&str; 9] = [
  "manifest.json",
  "paths.bin",
  "occ.bin",
  "symbols.bin",
  "external.bin",
  "texts.bin",
  "docs.bin",
  "refs.bin",
  "coverage.bin",
];

fn opened() -> (tempfile::TempDir, std::path::PathBuf, Shard) {
  let (root, dir) = support::root();
  Shard::build(&dir, &support::sample()).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  (root, dir, shard)
}

fn truncate(path: &std::path::Path, length: u64) {
  let file = std::fs::OpenOptions::new()
    .write(true)
    .open(path)
    .expect("open for truncate");
  file.set_len(length).expect("truncate");
}

#[test]
fn truncating_the_occurrence_table_is_reported_not_fatal() {
  let (_root, dir, shard) = opened();
  assert!(shard.definition_at("src/lib.rs", 12, 6).is_found());

  truncate(&dir.join("occ.bin"), 8);

  match shard.health().expect_err("shrunk table") {
    Error::Stale {
      table,
      expected,
      found,
    } => {
      assert_eq!(table, "occ.bin");
      assert!(expected > found, "{expected} vs {found}");
    }
    other => panic!("expected Stale, got {other}"),
  }
  // Queries must not touch the shrunk mapping; they degrade like an unopened
  // shard rather than reading out of range.
  assert_eq!(shard.definition_at("src/lib.rs", 12, 6), Answer::Uncovered);
  assert_eq!(shard.hover("src/lib.rs", 12, 6), Answer::Uncovered);
  assert!(shard.references(support::FOO).is_empty());
  assert_eq!(shard.coverage("src/lib.rs"), Coverage::Unknown);
  assert_eq!(shard.normalized_position("src/lib.rs", 20, 4), None);
  assert_eq!(shard.source_text("src/lib.rs"), None);
}

#[test]
fn truncating_a_documentation_blob_is_reported_too() {
  let (_root, dir, shard) = opened();
  assert!(shard.hover("src/lib.rs", 12, 6).is_found());

  truncate(&dir.join("docs.bin"), 8);

  assert!(matches!(
    shard.health().expect_err("shrunk docs"),
    Error::Stale {
      table: "docs.bin",
      ..
    }
  ));
  assert_eq!(shard.hover("src/lib.rs", 12, 6), Answer::Uncovered);
}

#[test]
fn a_truncated_shard_no_longer_opens() {
  let (_root, dir, _shard) = opened();
  truncate(&dir.join("occ.bin"), 8);
  assert!(Shard::open(&dir).expect("open").is_none());
  assert!(Shard::open_strict(&dir).is_err());
}

#[test]
fn replacing_the_files_keeps_serving_the_opened_snapshot() {
  let (_root, dir, shard) = opened();
  assert_eq!(shard.references(support::FOO).len(), 4);

  // A rebuild writes new files and renames them in, exactly like `Shard::build`.
  let (_other, replacement) = support::root();
  Shard::build(&replacement, &gity_scip::Decoded::new("2222222222222222")).expect("build small");
  for name in FILES {
    std::fs::rename(replacement.join(name), dir.join(name)).expect("rename");
  }

  // The open handle still reads the inodes it mapped: old snapshot, no mix.
  assert!(
    shard.health().is_ok(),
    "a rename must not look like a shrink"
  );
  assert_eq!(shard.manifest().commit, support::COMMIT);
  assert_eq!(shard.references(support::FOO).len(), 4);
  let Answer::Found(hover) = shard.hover("src/lib.rs", 12, 6) else {
    panic!("the old snapshot must still answer");
  };
  assert_eq!(hover.documentation, "The Foo type.");

  // A fresh open sees the new snapshot.
  let fresh = Shard::open(&dir).expect("open").expect("shard present");
  assert_eq!(fresh.manifest().commit, "2222222222222222");
  assert!(fresh.references(support::FOO).is_empty());
  assert_eq!(fresh.definition_at("src/lib.rs", 12, 6), Answer::Uncovered);
  assert_eq!(fresh.coverage("src/lib.rs"), Coverage::Unknown);
}
