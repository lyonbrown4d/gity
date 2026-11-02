use super::*;
use std::io::Write;

#[test]
fn job_payload_defaults_the_artifact_name() {
  let job: Job = serde_json::from_value(serde_json::json!({"commit_sha":"abc123"})).unwrap();
  assert_eq!(job.commit_sha, "abc123");
  assert!(job.artifact_name.is_empty());
}

#[test]
fn artifact_names_must_be_single_safe_segments() {
  assert!(check_name("code-index.msgpack.zst").is_ok());
  for invalid in ["", ".", "..", "a/b", "a\\b", "a\0b"] {
    assert!(check_name(invalid).is_err(), "accepted {invalid:?}");
  }
}

#[tokio::test]
async fn scan_source_encodes_the_commit_and_definitions() {
  let dir = tempfile::tempdir().unwrap();
  let compressed = dir.path().join("source.zip");
  let mut zip = zip::ZipWriter::new(std::fs::File::create(&compressed).unwrap());
  zip
    .start_file("src/lib.rs", zip::write::SimpleFileOptions::default())
    .unwrap();
  zip.write_all(b"pub fn answer() -> u32 { 42 }\n").unwrap();
  zip.finish().unwrap();
  let workspace = dir.path().join("source");
  std::fs::create_dir(&workspace).unwrap();
  let scanned = scan_source(compressed, workspace, "abc123".into())
    .await
    .unwrap();
  assert_eq!(scanned.files, 1);
  assert!(scanned.definitions >= 1);
  let decoded = gity_code_intel::decode(&scanned.bytes).unwrap();
  assert_eq!(decoded.commit, "abc123");
  assert_eq!(decoded.files.len(), 1);
}
