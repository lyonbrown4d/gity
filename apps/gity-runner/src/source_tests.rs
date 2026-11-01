use super::*;
use std::io::Cursor;
#[test]
fn archive_paths_cannot_escape_workspace() {
  let buffer = Cursor::new(Vec::new());
  let mut zip = zip::ZipWriter::new(buffer);
  zip
    .start_file("../escape", zip::write::SimpleFileOptions::default())
    .unwrap();
  zip.write_all(b"bad").unwrap();
  let bytes = zip.finish().unwrap().into_inner();
  let dir = tempfile::tempdir().unwrap();
  assert!(extract_reader(Cursor::new(bytes), dir.path()).is_err());
}

#[cfg(unix)]
#[test]
fn source_links_preserve_valid_targets_and_reject_indirect_escape() {
  let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
  let options = zip::write::SimpleFileOptions::default();
  archive.start_file("source/file", options).unwrap();
  archive.write_all(b"source").unwrap();
  archive.add_symlink("file", "source/file", options).unwrap();
  let dir = tempfile::tempdir().unwrap();
  extract_reader(
    Cursor::new(archive.finish().unwrap().into_inner()),
    dir.path(),
  )
  .unwrap();
  assert_eq!(std::fs::read(dir.path().join("file")).unwrap(), b"source");
  let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
  archive.add_symlink("alias", ".", options).unwrap();
  archive.add_symlink("escape", "alias/..", options).unwrap();
  let dir = tempfile::tempdir().unwrap();
  assert!(
    extract_reader(
      Cursor::new(archive.finish().unwrap().into_inner()),
      dir.path()
    )
    .is_err()
  );
}
