use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
  fs::{self, File, OpenOptions},
  io::{Read, Write},
  path::{Component, Path},
};

pub const VERSION: u32 = 1;

/// Docker quota device identities belong to the host, not to persisted data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TarPolicy {
  Directory,
  FileBind,
  DockerData,
}

impl TarPolicy {
  pub fn volume(config: &serde_json::Value, key: &str) -> Self {
    let docker = config["services"].as_object().is_some_and(|services| {
      services.values().any(|service| {
        let image = service["image"].as_str().unwrap_or("");
        let official = [
          "docker:",
          "docker.io/library/docker:",
          "registry-1.docker.io/library/docker:",
        ]
        .iter()
        .any(|prefix| {
          image.strip_prefix(prefix).is_some_and(|tag| {
            tag
              .split('@')
              .next()
              .unwrap_or("")
              .split('-')
              .any(|part| part == "dind")
          })
        });
        official
          && service["privileged"] == true
          && service["volumes"].as_array().is_some_and(|mounts| {
            mounts.iter().any(|mount| {
              mount["type"] == "volume"
                && mount["source"] == key
                && mount["target"] == "/var/lib/docker"
            })
          })
      })
    });
    if docker {
      Self::DockerData
    } else {
      Self::Directory
    }
  }

  pub fn excludes(self, path: &Path) -> bool {
    self == Self::DockerData
      && ["volumes/backingFsBlockDev", "overlay2/backingFsBlockDev"]
        .iter()
        .any(|value| {
          path.as_os_str() == std::ffi::OsStr::new(value)
            || path.as_os_str() == std::ffi::OsStr::new(&format!("./{value}"))
        })
  }

  pub fn tar_exclusions(self, command: &mut std::process::Command) {
    if self == Self::DockerData {
      // Moby v28.5.2 quota/projectquota.go makeBackingFsDev recreates these
      // using the new filesystem's device. Never recreate archived host devices.
      command.args([
        "--anchored",
        "--no-wildcards",
        "--exclude=./volumes/backingFsBlockDev",
        "--exclude=volumes/backingFsBlockDev",
        "--exclude=./overlay2/backingFsBlockDev",
        "--exclude=overlay2/backingFsBlockDev",
      ]);
    }
  }
}

#[cfg(test)]
#[path = "policy_tests.rs"]
mod policy_tests;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
  pub format_version: u32,
  pub source_project: String,
  pub created_at: String,
  pub consistency: String,
  pub compose: Archive,
  pub volumes: Vec<Volume>,
  pub binds: Vec<Bind>,
  pub images: Vec<Image>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archive {
  pub file: String,
  pub size: u64,
  pub sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Volume {
  pub key: String,
  pub source_name: String,
  /// None means the declared volume had never been created in the source project.
  pub archive: Option<Archive>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bind {
  pub id: String,
  pub source: String,
  pub directory: bool,
  pub archive: Archive,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
  pub service: String,
  pub reference: String,
  pub image_id: String,
}

pub fn validate_project(value: &str) -> Result<()> {
  if value.is_empty()
    || value.len() > 63
    || !value
      .bytes()
      .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    || !value.as_bytes()[0].is_ascii_alphanumeric()
  {
    bail!("project must use lowercase letters, digits, '-' or '_' and start with a letter/digit");
  }
  if value == "gity-rust-blackbox-20261101" {
    bail!("reserved retained acceptance project cannot be modified");
  }
  Ok(())
}

pub fn safe_component(value: &str) -> Result<()> {
  if value.is_empty()
    || !value
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    || value == "."
    || value == ".."
  {
    bail!("invalid snapshot path component");
  }
  Ok(())
}

pub fn safe_relative(path: &Path) -> Result<()> {
  if path.is_absolute()
    || path
      .components()
      .any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
  {
    bail!("archive contains an unsafe path");
  }
  Ok(())
}

pub fn create_private_dir(path: &Path) -> Result<()> {
  #[cfg(unix)]
  {
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
      .create(path)
      .context("create new private directory")?;
  }
  #[cfg(not(unix))]
  bail!("offline backup requires a Unix host to enforce private permissions");
  Ok(())
}

pub fn private_file(path: &Path) -> Result<File> {
  let mut options = OpenOptions::new();
  options.create_new(true).write(true);
  #[cfg(unix)]
  {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o600);
  }
  options.open(path).context("create new private bundle file")
}

pub fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
  let mut file = private_file(path)?;
  file.write_all(bytes)?;
  file.sync_all()?;
  Ok(())
}

pub fn identity(path: &Path) -> Result<Archive> {
  let mut reader = File::open(path)?;
  let mut hash = Sha256::new();
  let mut size = 0_u64;
  let mut block = [0_u8; 128 * 1024];
  loop {
    let count = reader.read(&mut block)?;
    if count == 0 {
      break;
    }
    hash.update(&block[..count]);
    size += count as u64;
  }
  Ok(Archive {
    file: path
      .file_name()
      .context("archive missing name")?
      .to_str()
      .context("archive name is not UTF-8")?
      .into(),
    size,
    sha256: hex::encode(hash.finalize()),
  })
}
