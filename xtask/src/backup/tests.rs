use super::{model, restore, verify};
use serde_json::json;
use std::{
  collections::{BTreeMap, BTreeSet},
  io::Write,
  path::{Path, PathBuf},
};

#[test]
fn rejects_unsafe_project_and_archive_components() {
  for value in [
    "",
    "../escape",
    "with space",
    "-leading",
    "gity-rust-blackbox-20261101",
  ] {
    assert!(model::validate_project(value).is_err(), "accepted {value}");
  }
  assert!(model::validate_project("gity-bb-target-1").is_ok());
  for value in ["", "..", ".", "../outside", "/absolute", "sub/path"] {
    assert!(model::safe_component(value).is_err(), "accepted {value}");
  }
  assert!(model::safe_relative(Path::new("safe/../outside")).is_err());
}

#[test]
fn corrupted_archive_is_rejected_before_restore_allocation() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("volume.tar");
  std::fs::write(&path, b"original offline payload").unwrap();
  let archive = model::identity(&path).unwrap();
  verify::verify_file(dir.path(), &archive, &mut BTreeSet::new()).unwrap();
  std::fs::OpenOptions::new()
    .append(true)
    .open(&path)
    .unwrap()
    .write_all(b"corruption")
    .unwrap();
  assert!(verify::verify_file(dir.path(), &archive, &mut BTreeSet::new()).is_err());
}

#[test]
fn tar_traversal_and_file_bind_extras_are_rejected() {
  let dir = tempfile::tempdir().unwrap();
  let archive = dir.path().join("bad.tar");
  let mut header = tar::Header::new_gnu();
  header.set_path("safe").unwrap();
  header.set_size(1);
  header.set_mode(0o600);
  header.as_mut_bytes()[..11].copy_from_slice(b"../outside\0");
  header.set_cksum();
  let mut file = std::fs::File::create(&archive).unwrap();
  file.write_all(header.as_bytes()).unwrap();
  file.write_all(&[0; 512]).unwrap();
  file.write_all(&[0; 1024]).unwrap();
  drop(file);
  assert!(verify::verify_tar(&archive, model::TarPolicy::Directory).is_err());
  let archive = dir.path().join("extra.tar");
  let mut builder = tar::Builder::new(std::fs::File::create(&archive).unwrap());
  let mut header = tar::Header::new_gnu();
  header.set_size(1);
  header.set_mode(0o600);
  header.set_cksum();
  builder
    .append_data(&mut header, "unexpected", &b"x"[..])
    .unwrap();
  builder.finish().unwrap();
  assert!(verify::verify_tar(&archive, model::TarPolicy::FileBind).is_err());
}

#[test]
fn remap_moves_runner_paths_and_isolates_project_networks() {
  let mut config = json!({
    "name":"source", "services": {"runner": {
      "build":{"context":"/original"}, "container_name":"source-runner",
      "command":["--work-root","/private/work"],
      "environment":{"GITY_COMPONENTS__RUNNER__WORK_ROOT":"/private/work"},
      "volumes":[{"type":"bind","source":"/private/work","target":"/private/work"}]
    }}, "networks":{"default":{"name":"source_default"}}
  });
  let binds = BTreeMap::from([(
    "/private/work".into(),
    PathBuf::from("/new-private/bind-0000"),
  )]);
  restore::remap(&mut config, "gity-bb-target", &binds).unwrap();
  assert_eq!(
    config["services"]["runner"]["command"][1],
    "/new-private/bind-0000"
  );
  assert_eq!(
    config["services"]["runner"]["environment"]["GITY_COMPONENTS__RUNNER__WORK_ROOT"],
    "/new-private/bind-0000"
  );
  assert_eq!(
    config["services"]["runner"]["volumes"][0]["target"],
    "/new-private/bind-0000"
  );
  assert_eq!(
    config["networks"]["default"]["name"],
    "gity-bb-target_default"
  );
  assert!(config["services"]["runner"].get("build").is_none());
  assert!(config["services"]["runner"].get("container_name").is_none());
}

#[cfg(unix)]
#[test]
fn bundles_use_private_host_permissions_and_reject_symlink_entries() {
  use std::os::unix::fs::{PermissionsExt, symlink};
  let parent = tempfile::tempdir().unwrap();
  let dir = parent.path().join("bundle");
  model::create_private_dir(&dir).unwrap();
  let file = dir.join("secret.json");
  model::write_private(&file, b"secret material").unwrap();
  assert_eq!(
    std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
    0o700
  );
  assert_eq!(
    std::fs::metadata(&file).unwrap().permissions().mode() & 0o777,
    0o600
  );
  let mut archive = model::identity(&file).unwrap();
  symlink(&file, dir.join("link.json")).unwrap();
  archive.file = "link.json".into();
  assert!(verify::verify_file(&dir, &archive, &mut BTreeSet::new()).is_err());
}

#[test]
fn file_bind_symlinks_cannot_redirect_a_restored_host_bind() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("symlink.tar");
  let mut builder = tar::Builder::new(std::fs::File::create(&path).unwrap());
  let mut header = tar::Header::new_gnu();
  header.set_size(0);
  header.set_mode(0o777);
  header.set_entry_type(tar::EntryType::Symlink);
  header.set_link_name("/etc/shadow").unwrap();
  header.set_cksum();
  builder
    .append_data(&mut header, "payload", std::io::empty())
    .unwrap();
  builder.finish().unwrap();
  assert!(verify::verify_tar(&path, model::TarPolicy::FileBind).is_err());
}

#[test]
fn volume_tar_allows_only_zero_device_whiteouts_and_file_binds_remain_regular() {
  let directory = tempfile::tempdir().unwrap();
  for (index, (kind, major, minor, size, expected)) in [
    (tar::EntryType::Char, 0, 0, 0, true),
    (tar::EntryType::Char, 1, 3, 0, false),
    (tar::EntryType::Char, 0, 1, 0, false),
    (tar::EntryType::Char, 1, 0, 0, false),
    (tar::EntryType::Char, 0, 0, 1, false),
    (tar::EntryType::Block, 0, 0, 0, false),
    (tar::EntryType::Fifo, 0, 0, 0, false),
  ]
  .into_iter()
  .enumerate()
  {
    let path = directory.path().join(format!("special-{index}.tar"));
    let mut builder = tar::Builder::new(std::fs::File::create(&path).unwrap());
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(kind);
    header.set_size(size);
    header.set_mode(0o600);
    header.set_device_major(major).unwrap();
    header.set_device_minor(minor).unwrap();
    header.set_cksum();
    let bytes = vec![0; size as usize];
    builder
      .append_data(&mut header, "payload", bytes.as_slice())
      .unwrap();
    builder.finish().unwrap();
    assert_eq!(
      verify::verify_tar(&path, model::TarPolicy::Directory).is_ok(),
      expected,
      "case {index}"
    );
    assert!(
      verify::verify_tar(&path, model::TarPolicy::FileBind).is_err(),
      "file bind case {index}"
    );
  }
}
