use super::*;

#[test]
fn parses_maven_release_and_snapshot_metadata_coordinates() {
  assert_eq!(
    coordinates("com/example/widget/maven-metadata.xml").unwrap(),
    ("com.example".to_owned(), "widget".to_owned(), None)
  );
  assert_eq!(
    coordinates("com/example/widget/1.2-SNAPSHOT/maven-metadata.xml").unwrap(),
    (
      "com.example".to_owned(),
      "widget".to_owned(),
      Some("1.2-SNAPSHOT".to_owned())
    )
  );
}

#[test]
fn writes_release_versions_and_snapshot_build_metadata() {
  let versions = vec![
    json!({"version":{"version":"1.0.0"},"files":[]}),
    json!({"version":{"version":"1.1-SNAPSHOT"},"files":[
        {"file_name":"widget-1.1-20261101.100000-2.jar"}
    ]}),
  ];
  let releases = artifact_metadata("com.example", "widget", &versions);
  assert!(releases.contains("<latest>1.1-SNAPSHOT</latest>"));
  assert!(releases.contains("<release>1.0.0</release>"));
  let snapshot = snapshot_metadata("com.example", "widget", "1.1-SNAPSHOT", &versions);
  assert!(snapshot.contains("<timestamp>20261101.100000</timestamp>"));
  assert!(snapshot.contains("<buildNumber>2</buildNumber>"));
  assert!(snapshot.contains("<value>1.1-20261101.100000-2</value>"));
}

#[test]
fn rejects_malformed_metadata_xml() {
  assert!(validate_metadata_document(b"<metadata>").is_err());
  assert!(validate_metadata_document(b"<metadata/>").is_ok());
}

#[test]
fn orders_release_metadata_by_numeric_version_components() {
  let versions = vec![
    json!({"version":{"version":"1.9.0","updated_at":"2026-11-01T05:00:00Z"},"files":[]}),
    json!({"version":{"version":"1.10.0","updated_at":"2026-11-01T06:00:00Z"},"files":[]}),
  ];
  let metadata = artifact_metadata("com.example", "widget", &versions);
  assert!(metadata.contains("<latest>1.10.0</latest>"));
  assert!(metadata.contains("<lastUpdated>20261101060000</lastUpdated>"));
  assert_eq!(
    metadata,
    artifact_metadata("com.example", "widget", &versions)
  );
}

#[test]
fn snapshot_metadata_selects_latest_build_and_ignores_sidecars() {
  let versions = vec![
    json!({"version":{"version":"1.1-SNAPSHOT","updated_at":"2026-11-02T09:00:00Z"},"files":[
      {"file_name":"widget-1.1-20261102.090000-1.jar","updated_at":"2026-11-02T09:00:01Z"},
      {"file_name":"widget-1.1-20261101.100000-9.jar","updated_at":"2026-11-01T10:00:01Z"},
      {"file_name":"widget-1.1-20261102.090000-1.jar.sha1","updated_at":"2026-11-02T09:00:01Z"},
      {"file_name":"widget-1.1-20261102.090000-1-sources.jar","updated_at":"2026-11-02T09:00:02Z"}
    ]}),
  ];
  let metadata = snapshot_metadata("com.example", "widget", "1.1-SNAPSHOT", &versions);
  assert!(metadata.contains("<timestamp>20261102.090000</timestamp>"));
  assert!(metadata.contains("<buildNumber>1</buildNumber>"));
  assert!(metadata.contains("<lastUpdated>20261102090000</lastUpdated>"));
  assert!(metadata.contains("<updated>20261102090001</updated>"));
  assert!(metadata.contains("<value>1.1-20261102.090000-1</value>"));
  assert!(!metadata.contains("<extension>sha1</extension>"));
  assert!(metadata.contains("<classifier>sources</classifier>"));
}

#[test]
fn recognizes_and_validates_metadata_checksum_paths() {
  assert_eq!(
    metadata_checksum_path("com/example/widget/maven-metadata.xml.sha1"),
    Some(("com/example/widget/maven-metadata.xml", ".sha1"))
  );
  assert!(validate_metadata_checksum(&[b'a'; 40], ".sha1").is_ok());
  assert!(validate_metadata_checksum(b"not-a-checksum", ".md5").is_err());
}
