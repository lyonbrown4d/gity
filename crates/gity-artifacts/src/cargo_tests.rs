use super::*;
use flate2::{Compression, write::GzEncoder};
use std::io::Cursor;

#[test]
fn parses_cargo_package_and_dependency_index_metadata() {
  let content = br#"[package]
name = "widgets"
version = "1.2.3"
rust-version = "1.75"
[dependencies]
serde = { version = "^1.0", features = ["derive"], optional = true }
[target.'cfg(unix)'.build-dependencies]
cc = "1"
[features]
default = ["serde"]
"#;
  let mut crate_bytes = Vec::new();
  {
    let encoder = GzEncoder::new(&mut crate_bytes, Compression::default());
    let mut archive = tar::Builder::new(encoder);
    let mut header = tar::Header::new_gnu();
    header.set_size(content.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    archive
      .append_data(
        &mut header,
        "widgets-1.2.3/Cargo.toml",
        Cursor::new(content),
      )
      .unwrap();
    archive.into_inner().unwrap().finish().unwrap();
  }
  let metadata = cargo_metadata(&crate_bytes).unwrap();
  assert_eq!(metadata.name, "widgets");
  assert_eq!(metadata.version, "1.2.3");
  assert_eq!(metadata.dependencies.len(), 2);
  assert_eq!(metadata.dependencies[0]["features"][0], "derive");
  assert_eq!(metadata.dependencies[1]["target"], "cfg(unix)");
}

#[test]
fn cargo_sparse_index_path_uses_length_buckets() {
  assert_eq!(cargo_index_path("a").unwrap(), "1/a");
  assert_eq!(cargo_index_path("ab").unwrap(), "2/ab");
  assert_eq!(cargo_index_path("abc").unwrap(), "3/a/abc");
  assert_eq!(cargo_index_path("serde_json").unwrap(), "se/rd/serde-json");
}
