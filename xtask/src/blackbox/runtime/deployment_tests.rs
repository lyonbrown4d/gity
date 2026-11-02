use super::{ASSET_PERMISSIONS, Mode, seaweed_owner};

#[test]
fn standalone_never_queries_or_reassigns_the_seaweed_runtime_identity() {
  assert_eq!(
    seaweed_owner(Mode::Standalone, || panic!(
      "Standalone must not query SeaweedFS"
    ))
    .unwrap(),
    None
  );
}

#[test]
fn distributed_uses_the_official_images_actual_numeric_uid_and_gid() {
  assert_eq!(
    seaweed_owner(Mode::Distributed, || Ok(b"1234\n5678\n".to_vec())).unwrap(),
    Some((1234, 5678))
  );
  // UID zero is still a numeric identity; never substitute an assumed service UID.
  assert_eq!(
    seaweed_owner(Mode::Distributed, || Ok(b"0\n0\n".to_vec())).unwrap(),
    Some((0, 0))
  );
  assert!(ASSET_PERMISSIONS.contains("chmod 0400 /run/secrets/seaweed-s3.json"));
  assert!(ASSET_PERMISSIONS.contains("chown \"$2:$3\" /run/secrets/seaweed-s3.json"));
}

#[test]
fn malformed_or_failed_identity_queries_refuse_permission_changes() {
  for value in [
    "",
    "1234\n",
    "1234\n5678\nextra",
    "1234\n5678\n\n",
    "-1\n42",
    "+1\n42",
    "1;bad\n42",
    "4294967296\n42",
    " 1234\n5678",
  ] {
    assert!(
      seaweed_owner(Mode::Distributed, || Ok(value.as_bytes().to_vec())).is_err(),
      "{value:?}"
    );
  }
  assert!(seaweed_owner(Mode::Distributed, || anyhow::bail!("image unavailable")).is_err());
}
