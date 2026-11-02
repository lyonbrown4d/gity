//! Opt-in Linux/Docker metadata roundtrip; permission failures are test failures.
use super::{docker, model::TarPolicy, ownership, restore, verify};
use anyhow::{Context, Result};
use serde_json::Value;

const HELPER: &str = "debian:bookworm-slim";
const PYTHON: &str = "python:3.13.2-bookworm";
const CREATE: &str = r#"
import os, pathlib, stat, struct
p = pathlib.Path('/data/value')
p.write_bytes(b'metadata payload\x00\xff')
os.chown(p, 12345, 23456)
os.chmod(p, 0o640)
os.setxattr(p, 'user.gity.snapshot', b'\x00\xffmetadata')
acl = struct.pack('<I', 2)
for tag, perm, uid in [(1,6,0xffffffff),(2,4,12346),(4,4,0xffffffff),(16,4,0xffffffff),(32,0,0xffffffff)]:
    acl += struct.pack('<HHI', tag, perm, uid)
os.setxattr(p, 'system.posix_acl_access', acl)
os.setxattr(p, 'security.capability', struct.pack('<5I', 0x02000001, 1 << 13, 0, 0, 0))
s = pathlib.Path('/data/sgid')
s.write_bytes(b'preserve SGID')
os.chown(s, 12345, 23456)
os.chmod(s, 0o2751)
os.mkdir('/data/opaque')
os.setxattr('/data/opaque', 'trusted.overlay.opaque', b'y')
os.mknod('/data/removed', stat.S_IFCHR | 0o000, os.makedev(0,0))
"#;
const CHECK: &str = r#"
import os, pathlib, stat, struct
p = pathlib.Path('/data/value')
assert p.read_bytes() == b'metadata payload\x00\xff'
assert (p.stat().st_uid, p.stat().st_gid) == (12345,23456)
assert os.getxattr(p, 'user.gity.snapshot') == b'\x00\xffmetadata'
acl = struct.pack('<I', 2)
for tag, perm, uid in [(1,6,0xffffffff),(2,4,12346),(4,4,0xffffffff),(16,4,0xffffffff),(32,0,0xffffffff)]:
    acl += struct.pack('<HHI', tag, perm, uid)
assert os.getxattr(p, 'system.posix_acl_access') == acl
assert os.getxattr(p, 'security.capability') == struct.pack('<5I', 0x02000001, 1 << 13, 0, 0, 0)
s = pathlib.Path('/data/sgid').stat()
assert (s.st_uid,s.st_gid,stat.S_IMODE(s.st_mode)) == (12345,23456,0o2751)
assert os.getxattr('/data/opaque', 'trusted.overlay.opaque') == b'y'
w = os.lstat('/data/removed')
assert stat.S_ISCHR(w.st_mode) and os.major(w.st_rdev) == 0 and os.minor(w.st_rdev) == 0
"#;

#[test]
#[ignore = "requires rootful Linux Docker, cached GNU tar/Python images and SYS_ADMIN/MKNOD metadata permissions"]
fn metadata_roundtrip_preserves_overlay_xattrs_acl_capabilities_sgid_whiteout_and_ownership()
-> Result<()> {
  let mut volumes = ScopedVolumes::new();
  let source = volumes.create("source")?;
  let target = volumes.create("target")?;
  docker::verify_helper(HELPER)?;
  python(&source, &format!("{CREATE}\n{CHECK}"))
    .context("metadata source fixture requires real xattr/capability/device permissions")?;
  let directory = tempfile::tempdir()?;
  let path = directory.path().join("metadata.tar");
  docker::archive(HELPER, "volume", &source, TarPolicy::Directory, &path)?;
  verify::verify_tar(&path, TarPolicy::Directory)?;
  restore::extract(HELPER, "volume", &target, &path, TarPolicy::Directory)?;
  python(&target, CHECK).context("restored metadata must match the source exactly")?;
  Ok(())
}

fn python(volume: &str, script: &str) -> Result<()> {
  let helper = docker::Helper::new(PYTHON);
  let result = helper
    .command()
    .args([
      "--cap-add",
      "SYS_ADMIN",
      "--cap-add",
      "SETFCAP",
      "--cap-add",
      "MKNOD",
      "--cap-add",
      "FSETID",
    ])
    .arg("--mount")
    .arg(docker::mount("volume", volume, "/data", false)?)
    .args([helper.image(), "python3", "-c", script])
    .output()
    .context("run metadata fixture/check helper")?;
  anyhow::ensure!(
    result.status.success(),
    "metadata helper failed: {}",
    String::from_utf8_lossy(&result.stderr)
  );
  Ok(())
}

/// Both volumes belong solely to this test's UUID invocation. Never discovers,
/// mounts, removes or reuses a user's retained Compose project data.
struct ScopedVolumes {
  project: String,
  invocation: String,
  volumes: Vec<(String, String)>,
}
impl ScopedVolumes {
  fn new() -> Self {
    Self {
      project: format!("gity-meta-{}", uuid::Uuid::now_v7().simple()),
      invocation: uuid::Uuid::now_v7().to_string(),
      volumes: Vec::new(),
    }
  }
  fn create(&mut self, key: &str) -> Result<String> {
    let name = ownership::create_volume(&self.project, key, &self.invocation)?;
    self.volumes.push((name.clone(), key.into()));
    Ok(name)
  }
}
impl Drop for ScopedVolumes {
  fn drop(&mut self) {
    for (name, key) in &self.volumes {
      let proof = docker::output(
        docker::command().args(["volume", "inspect", name]),
        "inspect metadata test volume ownership",
      )
      .ok()
      .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
      if proof.is_some_and(|value| {
        value[0]["Name"] == *name
          && value[0]["Labels"]["com.docker.compose.project"] == self.project
          && value[0]["Labels"]["com.docker.compose.volume"] == *key
          && value[0]["Labels"]["io.gity.backup.restore"] == self.invocation
      }) {
        if docker::status(
          docker::command().args(["volume", "rm", name]),
          "remove this invocation's metadata test volume",
        )
        .is_err()
        {
          eprintln!("Metadata test volume cleanup failed: {name}");
        }
      } else {
        eprintln!("Metadata test volume ownership unverified; preserved: {name}");
      }
    }
  }
}
