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

#[test]
fn public_key_parser_canonicalizes_comments_and_rejects_non_public_inputs() {
  let key = PrivateKey::random(&mut rand::rng(), russh::keys::Algorithm::Ed25519).unwrap();
  let public = key.public_key().to_openssh().unwrap();
  let (canonical, fingerprint) = canonical_key(&format!(" {public} workstation ")).unwrap();
  assert_eq!(canonical, public);
  assert_eq!(
    fingerprint,
    key.public_key().fingerprint(HashAlg::Sha256).to_string()
  );
  for invalid in [
    format!("{public}\n{public}"),
    format!("command=\"anything\" {public}"),
    key.to_openssh(LineEnding::LF).unwrap().to_string(),
    "ssh-ed25519 invalid".into(),
    "x".repeat(16385),
  ] {
    assert!(canonical_key(&invalid).is_err());
  }
}
