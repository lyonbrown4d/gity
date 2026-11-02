use super::SummaryExport;

#[test]
fn preserves_existing_summary_instead_of_overwriting_it() {
  let directory = tempfile::tempdir().unwrap();
  let destination = directory.path().join("existing.json");
  std::fs::write(&destination, b"existing report").unwrap();
  assert!(SummaryExport::prepare(directory.path(), &destination).is_err());
  assert_eq!(std::fs::read(destination).unwrap(), b"existing report");
}

#[test]
fn publishes_json_from_isolated_scratch_with_private_permissions() {
  let directory = tempfile::tempdir().unwrap();
  let destination = directory.path().join("result.json");
  let mut export = SummaryExport::prepare(directory.path(), &destination).unwrap();
  std::fs::write(
    export.output_path(),
    br#"{"metrics":{"requests":{"count":4}}}"#,
  )
  .unwrap();
  assert!(export.publish().unwrap());
  drop(export);
  let value: serde_json::Value =
    serde_json::from_slice(&std::fs::read(&destination).unwrap()).unwrap();
  assert_eq!(value["metrics"]["requests"]["count"], 4);
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
      std::fs::metadata(destination).unwrap().permissions().mode() & 0o777,
      0o600
    );
  }
}

#[test]
fn invalid_or_missing_output_leaves_no_empty_report() {
  let directory = tempfile::tempdir().unwrap();
  let destination = directory.path().join("result.json");
  {
    let mut export = SummaryExport::prepare(directory.path(), &destination).unwrap();
    assert!(!export.publish().unwrap());
    std::fs::write(export.output_path(), b"not JSON").unwrap();
    assert!(export.publish().is_err());
  }
  assert!(!destination.exists());
}

#[cfg(unix)]
#[test]
fn never_removes_a_replacement_at_the_reserved_path() {
  use std::os::unix::fs::symlink;
  let directory = tempfile::tempdir().unwrap();
  let destination = directory.path().join("result.json");
  let export = SummaryExport::prepare(directory.path(), &destination).unwrap();
  std::fs::remove_file(&destination).unwrap();
  let protected = directory.path().join("protected.json");
  std::fs::write(&protected, b"keep").unwrap();
  symlink(&protected, &destination).unwrap();
  drop(export);
  assert!(
    std::fs::symlink_metadata(destination)
      .unwrap()
      .file_type()
      .is_symlink()
  );
  assert_eq!(std::fs::read(protected).unwrap(), b"keep");
}
