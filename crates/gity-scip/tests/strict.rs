//! `open_strict` names the reason a shard is unusable; `open` stays silent.

mod support;

use gity_scip::{Error, FORMAT_VERSION, Shard};
use std::path::Path;

fn build() -> (tempfile::TempDir, std::path::PathBuf) {
  let (root, dir) = support::root();
  Shard::build(&dir, &support::sample()).expect("build");
  (root, dir)
}

#[test]
fn a_missing_shard_is_not_an_error() {
  let (root, dir) = support::root();
  assert!(Shard::open_strict(&dir).expect("missing").is_none());
  assert!(Shard::open(&dir).expect("open").is_none());
  drop(root);
}

#[test]
fn a_broken_manifest_is_reported() {
  let (_root, dir) = build();
  std::fs::write(dir.join("manifest.json"), b"{\"format_version\": ").expect("write");
  let error = Shard::open_strict(&dir).expect_err("broken manifest");
  assert!(matches!(error, Error::Manifest(_)), "{error}");
  assert!(Shard::open(&dir).expect("open").is_none());
}

#[test]
fn a_foreign_format_version_is_reported() {
  let (_root, dir) = build();
  let text = std::fs::read_to_string(dir.join("manifest.json")).expect("read");
  let bumped = text.replace(
    &format!("\"format_version\": {FORMAT_VERSION}"),
    "\"format_version\": 99",
  );
  assert_ne!(bumped, text, "manifest field spelling changed");
  std::fs::write(dir.join("manifest.json"), bumped).expect("write");
  match Shard::open_strict(&dir).expect_err("version mismatch") {
    Error::Version { found, expected } => {
      assert_eq!(found, 99);
      assert_eq!(expected, FORMAT_VERSION);
    }
    other => panic!("expected Version, got {other}"),
  }
  assert!(Shard::open(&dir).expect("open").is_none());
}

#[test]
fn a_missing_table_is_reported_with_its_path() {
  let (_root, dir) = build();
  std::fs::remove_file(dir.join("occ.bin")).expect("remove");
  match Shard::open_strict(&dir).expect_err("missing table") {
    Error::Io { path, .. } => assert!(path.ends_with("occ.bin"), "{path}"),
    other => panic!("expected Io, got {other}"),
  }
  assert!(Shard::open(&dir).expect("open").is_none());
}

#[test]
fn a_truncated_table_is_reported() {
  let (_root, dir) = build();
  std::fs::write(dir.join("occ.bin"), b"GSO1\x01\0\0\0").expect("truncate");
  match Shard::open_strict(&dir).expect_err("truncated table") {
    Error::Corrupt { table } => assert_eq!(table, "occ.bin"),
    other => panic!("expected Corrupt, got {other}"),
  }
  assert!(Shard::open(&dir).expect("open").is_none());
}

#[test]
fn a_foreign_table_tag_is_reported() {
  let (_root, dir) = build();
  let mut bytes = std::fs::read(dir.join("paths.bin")).expect("read");
  bytes[..4].copy_from_slice(b"XXXX");
  std::fs::write(dir.join("paths.bin"), bytes).expect("write");
  assert!(matches!(
    Shard::open_strict(&dir).expect_err("wrong tag"),
    Error::Corrupt { table } if table == "paths.bin"
  ));
}

#[test]
fn tables_that_disagree_are_reported() {
  let (_root, dir) = build();
  // Claim zero symbols in refs.bin: its CSR can no longer match symbols.bin.
  let mut refs = std::fs::read(dir.join("refs.bin")).expect("read");
  refs[4..8].copy_from_slice(&0u32.to_le_bytes());
  std::fs::write(dir.join("refs.bin"), refs).expect("write");
  assert!(matches!(
    Shard::open_strict(&dir).expect_err("inconsistent"),
    Error::Inconsistent
  ));
  assert!(Shard::open(&dir).expect("open").is_none());
}

#[test]
fn a_healthy_shard_opens_through_both_entry_points() {
  let (_root, dir) = build();
  let strict = Shard::open_strict(Path::new(&dir))
    .expect("strict open")
    .expect("shard present");
  let lenient = Shard::open(&dir).expect("open").expect("shard present");
  assert_eq!(strict.manifest(), lenient.manifest());
  assert_eq!(strict.references(support::FOO).len(), 4);
}
