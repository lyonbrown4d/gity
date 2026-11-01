use super::*;
#[test]
fn host_key_is_persisted_and_reused_without_allowing_public_file_permissions() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("host_key");
  let original = host_key(&path).unwrap();
  let loaded = host_key(&path).unwrap();
  assert_eq!(original.public_key(), loaded.public_key());
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
      std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
      0o600
    );
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(host_key(&path).is_err());
    let link = dir.path().join("symlink");
    std::os::unix::fs::symlink(&path, &link).unwrap();
    assert!(host_key(&link).is_err());
  }
}
