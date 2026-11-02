//! Shard lifecycle: writing files, reopening them, and ignoring unusable ones.

mod support;

use gity_scip::{Counts, CoverageSummary, Error, FORMAT_VERSION, Shard};
use std::path::Path;

const TABLES: [&str; 6] = [
  "paths.bin",
  "occ.bin",
  "symbols.bin",
  "docs.bin",
  "refs.bin",
  "coverage.bin",
];

#[test]
fn open_reports_nothing_when_no_shard_was_written() {
  let (_root, shard) = support::root();
  assert!(
    Shard::open(&shard)
      .expect("open missing directory")
      .is_none()
  );
  assert!(
    Shard::open(shard.parent().expect("parent"))
      .expect("open empty")
      .is_none()
  );
}

#[test]
fn build_writes_every_table_and_the_manifest() {
  let (_root, dir) = support::root();
  let manifest = Shard::build(&dir, &support::sample()).expect("build");
  assert_eq!(manifest.format_version, FORMAT_VERSION);
  assert_eq!(manifest.commit, support::COMMIT);
  assert_eq!(manifest.indexer, "rust-analyzer");
  assert_eq!(manifest.indexer_version, "2026-10-01");
  assert_eq!(
    manifest.counts,
    Counts {
      paths: 3,
      occurrences: 6,
      symbols: 2,
      references: 5,
      doc_bytes: "The Foo type.struct Foo".len(),
    }
  );
  assert_eq!(
    manifest.coverage,
    CoverageSummary {
      total_paths: 3,
      precise_paths: 2,
      listed_paths: 1,
    }
  );
  assert!(!manifest.indexed_at.is_empty());
  for name in TABLES.iter().chain(["manifest.json"].iter()) {
    assert!(dir.join(name).is_file(), "{name} missing");
  }
  // The manifest is the commit point and round-trips through open().
  let reopened = Shard::open(&dir).expect("open").expect("shard present");
  assert_eq!(reopened.manifest(), &manifest);
  assert_eq!(reopened.dir(), dir.as_path());
}

#[test]
fn build_replaces_the_previous_snapshot() {
  let (_root, dir) = support::root();
  Shard::build(&dir, &support::sample()).expect("first build");
  let smaller = gity_scip::Decoded::new("1111111111111111");
  let manifest = Shard::build(&dir, &smaller).expect("second build");
  assert_eq!(manifest.commit, "1111111111111111");
  assert_eq!(manifest.counts.paths, 0);
  let reopened = Shard::open(&dir).expect("open").expect("shard present");
  assert_eq!(reopened.manifest().commit, "1111111111111111");
}

#[test]
fn open_ignores_a_foreign_manifest() {
  let (_root, dir) = support::root();
  std::fs::create_dir_all(&dir).expect("mkdir");
  std::fs::write(dir.join("manifest.json"), b"{\"format_version\": ").expect("write");
  assert!(
    Shard::open(&dir)
      .expect("open truncated manifest")
      .is_none()
  );
  std::fs::write(dir.join("manifest.json"), b"{}").expect("write");
  assert!(
    Shard::open(&dir)
      .expect("open incomplete manifest")
      .is_none()
  );
}

#[test]
fn open_ignores_an_unknown_format_version() {
  let (_root, dir) = support::root();
  let manifest = Shard::build(&dir, &support::sample()).expect("build");
  let text = std::fs::read_to_string(dir.join("manifest.json")).expect("read");
  let bumped = text.replace(
    &format!("\"format_version\": {FORMAT_VERSION}"),
    "\"format_version\": 99",
  );
  assert_ne!(bumped, text, "manifest field spelling changed");
  std::fs::write(dir.join("manifest.json"), bumped).expect("write");
  assert_eq!(manifest.format_version, FORMAT_VERSION);
  assert!(Shard::open(&dir).expect("open future shard").is_none());
}

#[test]
fn open_ignores_a_truncated_table() {
  let (_root, dir) = support::root();
  Shard::build(&dir, &support::sample()).expect("build");
  std::fs::write(dir.join("occ.bin"), b"GS").expect("truncate");
  assert!(Shard::open(&dir).expect("open truncated").is_none());
}

#[test]
fn open_ignores_tables_that_disagree_with_the_manifest() {
  let (_root, dir) = support::root();
  Shard::build(&dir, &support::sample()).expect("build");
  // Drop the last path's occurrence range by rewriting the path table count.
  let paths = std::fs::read(dir.join("paths.bin")).expect("read paths");
  std::fs::write(dir.join("paths.bin"), &paths[..paths.len() - 4]).expect("corrupt");
  assert!(Shard::open(&dir).expect("open inconsistent").is_none());
}

#[test]
fn build_refuses_duplicate_document_paths() {
  let (_root, dir) = support::root();
  let mut decoded = support::sample();
  let duplicate = decoded.documents[0].clone();
  decoded.documents.push(duplicate);
  assert!(matches!(
    Shard::build(&dir, &decoded),
    Err(Error::DuplicatePath(path)) if path == "src/lib.rs"
  ));
}

#[test]
fn build_refuses_paths_that_are_not_repository_relative() {
  let (_root, dir) = support::root();
  let mut decoded = support::sample();
  decoded.documents[0].path = "/etc/passwd".to_string();
  assert!(matches!(Shard::build(&dir, &decoded), Err(Error::Path(_))));
  let mut escaped = support::sample();
  escaped.documents[0].path = "../outside.rs".to_string();
  assert!(matches!(Shard::build(&dir, &escaped), Err(Error::Path(_))));
}

#[test]
fn build_accepts_a_leading_dot_slash() {
  let (_root, dir) = support::root();
  let mut decoded = support::sample();
  decoded.documents[0].path = "./src/lib.rs".to_string();
  let manifest = Shard::build(&dir, &decoded).expect("build");
  let shard = Shard::open(Path::new(&dir)).expect("open").expect("shard");
  assert_eq!(manifest.counts.paths, 3);
  assert!(shard.coverage("src/lib.rs").is_precise());
}
