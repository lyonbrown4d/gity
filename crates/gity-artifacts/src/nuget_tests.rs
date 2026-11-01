use super::*;
use std::io::{Cursor, Write};
use zip::{ZipWriter, write::SimpleFileOptions};

#[test]
fn extracts_identity_from_nupkg_nuspec() {
  let mut bytes = Cursor::new(Vec::new());
  {
    let mut archive = ZipWriter::new(&mut bytes);
    archive
      .start_file("Widget.nuspec", SimpleFileOptions::default())
      .unwrap();
    archive.write_all(br#"<package><metadata><id>Widget</id><version>1.2.3</version><dependencies><group targetFramework="net8.0"><dependency id="Logging" version="[2.1.0,3.0.0)" /></group></dependencies></metadata></package>"#).unwrap();
    archive.finish().unwrap();
  }
  let metadata = nuget_metadata(bytes.get_ref()).unwrap();
  assert_eq!(metadata.id, "Widget");
  assert_eq!(metadata.version, "1.2.3");
  assert_eq!(
    metadata.dependency_groups[0].target_framework.as_deref(),
    Some("net8.0")
  );
  assert_eq!(metadata.dependency_groups[0].dependencies[0].id, "Logging");
  assert_eq!(
    metadata.dependency_groups[0].dependencies[0].range,
    "[2.1.0,3.0.0)"
  );
}

#[test]
fn rejects_nupkg_without_manifest() {
  let mut bytes = Cursor::new(Vec::new());
  {
    let mut archive = ZipWriter::new(&mut bytes);
    archive
      .start_file("content.txt", SimpleFileOptions::default())
      .unwrap();
    archive.write_all(b"x").unwrap();
    archive.finish().unwrap();
  }
  assert!(nuget_metadata(bytes.get_ref()).is_err());
}
