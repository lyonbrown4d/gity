use super::*;

#[test]
fn validates_language_versions_and_generic_coordinates() {
  assert!(PackageFormat::Cargo.validate_version("1.2.3").is_ok());
  assert!(PackageFormat::Cargo.validate_version("1.2").is_err());
  assert!(PackageFormat::Pypi.validate_version("1.0rc1").is_ok());
  assert!(
    PackageFormat::Pypi
      .validate_version("not a version")
      .is_err()
  );
  assert!(
    PackageFormat::Maven
      .validate_version("1.0-SNAPSHOT")
      .is_ok()
  );

  let coordinate = ArtifactCoordinate::new(
    17,
    PackageFormat::Generic,
    "widget",
    "1.0",
    "dist/linux/widget.tar.gz",
  )
  .unwrap();
  assert_eq!(coordinate.path, "dist/linux/widget.tar.gz");
}

#[test]
fn rejects_traversal_and_absolute_artifact_paths() {
  for path in [
    "../secret",
    "dist/../../secret",
    "/absolute",
    "a//b",
    "a\\b",
  ] {
    assert!(ArtifactCoordinate::new(17, PackageFormat::Generic, "name", "1", path).is_err());
  }
}
