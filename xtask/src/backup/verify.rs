use super::{
  docker,
  model::{self, Archive, Manifest},
};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::{collections::BTreeSet, fs::File, path::Path};

pub fn bundle(path: &Path) -> Result<(Manifest, Value)> {
  let metadata = std::fs::symlink_metadata(path.join("manifest.json"))?;
  if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 4 * 1024 * 1024 {
    bail!("manifest must be a bounded regular file");
  }
  let manifest: Manifest = serde_json::from_reader(File::open(path.join("manifest.json"))?)?;
  if manifest.format_version != model::VERSION
    || manifest.consistency != "offline-all-project-containers-stopped"
  {
    bail!("unsupported backup format or consistency contract");
  }
  model::validate_project(&manifest.source_project)?;
  let mut files = BTreeSet::new();
  verify_file(path, &manifest.compose, &mut files)?;
  if manifest.compose.file != "compose.json" {
    bail!("unexpected Compose snapshot name");
  }
  let config: Value = serde_json::from_reader(File::open(path.join(&manifest.compose.file))?)?;
  let mut keys = BTreeSet::new();
  let mut names = BTreeSet::new();
  for volume in &manifest.volumes {
    model::safe_component(&volume.key)?;
    model::safe_component(&volume.source_name)?;
    if !names.insert(volume.source_name.clone())
      || !keys.insert(volume.key.clone())
      || !volume
        .source_name
        .starts_with(&format!("{}_", manifest.source_project))
    {
      bail!("invalid or duplicate volume mapping");
    }
    if config["volumes"][&volume.key]["name"] != volume.source_name {
      bail!("volume mapping differs from Compose snapshot");
    }
    if let Some(archive) = &volume.archive {
      verify_file(path, archive, &mut files)?;
      verify_tar(
        &path.join(&archive.file),
        model::TarPolicy::volume(&config, &volume.key),
      )?;
    }
  }
  if config["volumes"].as_object().map_or(0, |v| v.len()) != keys.len() {
    bail!("incomplete volume manifest");
  }
  let mut binds = BTreeSet::new();
  let mut ids = BTreeSet::new();
  for bind in &manifest.binds {
    model::safe_component(&bind.id)?;
    if !ids.insert(bind.id.clone()) || !binds.insert(bind.source.clone()) {
      bail!("duplicate bind mapping");
    }
    if !Path::new(&bind.source).is_absolute() {
      bail!("bind source identity must be absolute");
    }
    verify_file(path, &bind.archive, &mut files)?;
    verify_tar(
      &path.join(&bind.archive.file),
      if bind.directory {
        model::TarPolicy::Directory
      } else {
        model::TarPolicy::FileBind
      },
    )?;
  }
  let services = config["services"]
    .as_object()
    .context("Compose services missing")?;
  for service in services.values() {
    for mount in service["volumes"].as_array().into_iter().flatten() {
      if mount["type"] == "bind" && !mount["source"].as_str().is_some_and(|s| binds.contains(s)) {
        bail!("incomplete bind manifest");
      }
    }
  }
  let mut images = BTreeSet::new();
  for image in &manifest.images {
    if !images.insert(&image.service)
      || config["services"][&image.service]["image"] != image.reference
    {
      bail!("image manifest differs from Compose snapshot");
    }
    let inspect: Value = serde_json::from_slice(&docker::output(
      docker::command().args(["image", "inspect", &image.reference]),
      "verify required restore image",
    )?)?;
    if inspect[0]["Id"] != image.image_id {
      bail!("restore image identity differs from offline snapshot");
    }
  }
  if images.len() != services.len() {
    bail!("incomplete image manifest");
  }
  Ok((manifest, config))
}

pub fn verify_file(root: &Path, archive: &Archive, seen: &mut BTreeSet<String>) -> Result<()> {
  model::safe_component(&archive.file)?;
  if !seen.insert(archive.file.clone()) {
    bail!("duplicate archive filename");
  }
  let path = root.join(&archive.file);
  let metadata = std::fs::symlink_metadata(&path)?;
  if !metadata.is_file() || metadata.file_type().is_symlink() {
    bail!("bundle entries must be regular files");
  }
  let actual = model::identity(&path)?;
  if actual.size != archive.size || actual.sha256 != archive.sha256 {
    bail!("bundle size/checksum mismatch");
  }
  Ok(())
}

pub fn verify_tar(path: &Path, policy: model::TarPolicy) -> Result<()> {
  let mut archive = tar::Archive::new(File::open(path)?);
  let file_bind = policy == model::TarPolicy::FileBind;
  let mut count = 0;
  for entry in archive.entries()? {
    let entry = entry.context("read archive header")?;
    let path = entry.path()?;
    model::safe_relative(&path)?;
    if file_bind && path != Path::new("payload") {
      bail!("file bind archive contains unexpected entries");
    }
    let kind = entry.header().entry_type();
    if policy.excludes(&path) {
      if !kind.is_block_special()
        || entry.header().size()? != 0
        || entry.header().uid()? != 0
        || entry.header().gid()? != 0
      {
        bail!("Docker quota marker must be an empty root-owned block device");
      }
      // This exact host-specific marker is excluded by extraction too. It is
      // never materialized in the new volume, unlike allowed 0:0 whiteouts.
      continue;
    }
    if file_bind && !kind.is_file() {
      bail!("file bind archive must contain a regular file");
    }
    // Docker overlay2 layers use character device 0:0 for deleted entries.
    // Preserve only this inert whiteout marker, never general devices or FIFOs.
    let whiteout = kind.is_character_special()
      && entry.header().device_major()? == Some(0)
      && entry.header().device_minor()? == Some(0)
      && entry.header().size()? == 0;
    if !(kind.is_file() || kind.is_dir() || kind.is_symlink() || kind.is_hard_link() || whiteout) {
      bail!("archive contains unsupported special files");
    }
    if kind.is_hard_link() {
      model::safe_relative(&entry.link_name()?.context("hard link target missing")?)?;
    }
    // Container-layer volumes can legitimately contain absolute symlinks. They are
    // preserved only inside a helper with a read-only root and one new data mount;
    // archives are never unpacked by a host-side tar process.
    count += 1;
  }
  if count == 0 || (file_bind && count != 1) {
    bail!("empty or invalid file bind archive");
  }
  Ok(())
}
