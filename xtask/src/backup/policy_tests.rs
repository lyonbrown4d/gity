use super::TarPolicy;
use crate::backup::{model, verify};
use serde_json::json;
use std::{collections::BTreeSet, path::Path};

#[test]
fn docker_data_requires_official_dind_privilege_and_exact_volume_target() {
  let config = json!({"services":{"docker": {
    "image":"docker:28.5-dind", "privileged":true,
    "volumes":[{"type":"volume","source":"docker-data","target":"/var/lib/docker"}]
  }}});
  assert_eq!(
    TarPolicy::volume(&config, "docker-data"),
    TarPolicy::DockerData
  );
  assert_eq!(TarPolicy::volume(&config, "other"), TarPolicy::Directory);
  for (field, value) in [
    ("image", json!("untrusted/docker:28.5-dind")),
    ("image", json!("docker:28.5-cli")),
    ("privileged", json!(false)),
    (
      "volumes",
      json!([{"type":"bind","source":"docker-data","target":"/var/lib/docker"}]),
    ),
    (
      "volumes",
      json!([{"type":"volume","source":"docker-data","target":"/data"}]),
    ),
  ] {
    let mut altered = config.clone();
    altered["services"]["docker"][field] = value;
    assert_eq!(
      TarPolicy::volume(&altered, "docker-data"),
      TarPolicy::Directory,
      "{field}"
    );
  }
}

#[test]
fn only_exact_host_quota_markers_are_excluded() {
  for path in [
    "volumes/backingFsBlockDev",
    "./volumes/backingFsBlockDev",
    "overlay2/backingFsBlockDev",
  ] {
    assert!(TarPolicy::DockerData.excludes(Path::new(path)));
    assert!(!TarPolicy::Directory.excludes(Path::new(path)));
    assert!(!TarPolicy::FileBind.excludes(Path::new(path)));
  }
  for path in [
    "/volumes/backingFsBlockDev",
    "volumes//backingFsBlockDev",
    "volumes/./backingFsBlockDev",
    "volumes/backingFsBlockDev-extra",
    "container/backingFsBlockDev",
    "../volumes/backingFsBlockDev",
  ] {
    assert!(!TarPolicy::DockerData.excludes(Path::new(path)), "{path}");
  }
}

#[test]
fn quota_marker_is_only_a_skipped_empty_root_owned_block_node() {
  let directory = tempfile::tempdir().unwrap();
  for (index, (kind, size, uid, gid, name, accepted)) in [
    (
      tar::EntryType::Block,
      0,
      0,
      0,
      "./volumes/backingFsBlockDev",
      true,
    ),
    (
      tar::EntryType::Block,
      0,
      0,
      0,
      "volumes/backingFsBlockDev",
      true,
    ),
    (
      tar::EntryType::Block,
      0,
      0,
      0,
      "overlay2/backingFsBlockDev",
      true,
    ),
    (
      tar::EntryType::Block,
      1,
      0,
      0,
      "volumes/backingFsBlockDev",
      false,
    ),
    (
      tar::EntryType::Block,
      0,
      1,
      0,
      "volumes/backingFsBlockDev",
      false,
    ),
    (
      tar::EntryType::Block,
      0,
      0,
      1,
      "volumes/backingFsBlockDev",
      false,
    ),
    (
      tar::EntryType::Char,
      0,
      0,
      0,
      "volumes/backingFsBlockDev",
      false,
    ),
    (
      tar::EntryType::Fifo,
      0,
      0,
      0,
      "volumes/backingFsBlockDev",
      false,
    ),
    (
      tar::EntryType::Regular,
      0,
      0,
      0,
      "volumes/backingFsBlockDev",
      false,
    ),
    (
      tar::EntryType::Block,
      0,
      0,
      0,
      "volumes/not-the-marker",
      false,
    ),
  ]
  .into_iter()
  .enumerate()
  {
    let path = directory.path().join(format!("quota-{index}.tar"));
    let mut builder = tar::Builder::new(std::fs::File::create(&path).unwrap());
    let mut root = tar::Header::new_gnu();
    root.set_entry_type(tar::EntryType::Directory);
    root.set_size(0);
    root.set_mode(0o700);
    root.set_cksum();
    builder
      .append_data(&mut root, ".", std::io::empty())
      .unwrap();
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(kind);
    header.set_size(size);
    header.set_mode(0o600);
    header.set_uid(uid);
    header.set_gid(gid);
    header.set_device_major(259).unwrap();
    header.set_device_minor(2).unwrap();
    header.set_cksum();
    builder
      .append_data(&mut header, name, vec![0; size as usize].as_slice())
      .unwrap();
    builder.finish().unwrap();
    assert_eq!(
      verify::verify_tar(&path, TarPolicy::DockerData).is_ok(),
      accepted,
      "case {index}"
    );
    if kind == tar::EntryType::Block || kind == tar::EntryType::Fifo || kind == tar::EntryType::Char
    {
      assert!(
        verify::verify_tar(&path, TarPolicy::Directory).is_err(),
        "case {index}"
      );
    }
    assert!(
      verify::verify_tar(&path, TarPolicy::FileBind).is_err(),
      "case {index}"
    );
  }
}

#[test]
#[ignore = "requires GITY_BACKUP_VERIFY_BUNDLE pointing at a real private offline bundle; read-only"]
fn existing_bundle_checksums_and_archive_policies_verify_without_docker() -> anyhow::Result<()> {
  let root = std::path::PathBuf::from(std::env::var("GITY_BACKUP_VERIFY_BUNDLE")?);
  let manifest: model::Manifest =
    serde_json::from_reader(std::fs::File::open(root.join("manifest.json"))?)?;
  let config: serde_json::Value =
    serde_json::from_reader(std::fs::File::open(root.join("compose.json"))?)?;
  let mut seen = BTreeSet::new();
  verify::verify_file(&root, &manifest.compose, &mut seen)?;
  for volume in &manifest.volumes {
    if let Some(archive) = &volume.archive {
      verify::verify_file(&root, archive, &mut seen)?;
      verify::verify_tar(
        &root.join(&archive.file),
        TarPolicy::volume(&config, &volume.key),
      )?;
    }
  }
  for bind in &manifest.binds {
    verify::verify_file(&root, &bind.archive, &mut seen)?;
    verify::verify_tar(
      &root.join(&bind.archive.file),
      if bind.directory {
        TarPolicy::Directory
      } else {
        TarPolicy::FileBind
      },
    )?;
  }
  Ok(())
}
