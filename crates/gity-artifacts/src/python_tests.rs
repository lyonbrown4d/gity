use super::*;
use std::io::{Cursor, Write};
use zip::{ZipWriter, write::SimpleFileOptions};

#[test]
fn reads_wheel_metadata_and_python_dependencies() {
  let mut bytes = Cursor::new(Vec::new());
  {
    let mut archive = ZipWriter::new(&mut bytes);
    archive
      .start_file("demo-1.2.dist-info/METADATA", SimpleFileOptions::default())
      .unwrap();
    archive.write_all(b"Metadata-Version: 2.1\nName: Demo_Pkg\nVersion: 1.2rc1\nRequires-Python: >=3.10\nRequires-Dist: requests >= 2\n\n").unwrap();
    archive.finish().unwrap();
  }
  let metadata = python_metadata("demo.whl", bytes.get_ref()).unwrap();
  assert_eq!(metadata.name, "demo-pkg");
  assert_eq!(metadata.version, "1.2rc1");
  assert_eq!(metadata.requires_python.as_deref(), Some(">=3.10"));
  assert_eq!(metadata.requires_dist, ["requests >= 2"]);
}

#[test]
fn rejects_unrecognized_python_artifacts() {
  assert!(python_metadata("package.txt", b"not a package").is_err());
}
