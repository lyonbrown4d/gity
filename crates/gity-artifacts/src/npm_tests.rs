use super::*;
use flate2::{Compression, write::GzEncoder};
use std::io::Cursor;

#[test]
fn parses_npm_package_manifest_without_extracting_paths() {
  let mut compressed = Vec::new();
  {
    let encoder = GzEncoder::new(&mut compressed, Compression::default());
    let mut archive = tar::Builder::new(encoder);
    let manifest =
      br#"{"name":"@scope/widget","version":"1.2.3","dependencies":{"left-pad":"^1.3.0"}}"#;
    let mut header = tar::Header::new_gnu();
    header.set_size(manifest.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    archive
      .append_data(&mut header, "package/package.json", Cursor::new(manifest))
      .unwrap();
    archive.into_inner().unwrap().finish().unwrap();
  }
  let manifest = npm_manifest(&compressed).unwrap();
  assert_eq!(manifest["name"], "@scope/widget");
  assert_eq!(manifest["dependencies"]["left-pad"], "^1.3.0");
}

#[test]
fn sha256_integrity_uses_subresource_integrity_encoding() {
  let value =
    sha256_integrity("0967115f2813a3541eaef77de9d9d5773f1c0c04314b0bbfe4ff3b3b1c55b5d5").unwrap();
  assert!(value.starts_with("sha256-"));
  assert!(sha256_integrity("xyz").is_err());
}
