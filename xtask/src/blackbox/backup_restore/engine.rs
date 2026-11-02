//! Hosted bytes remain in Angos even after the public project policy becomes mirror.
use super::{
  probes::{self, Fingerprint},
  runtime::Restored,
};
use crate::blackbox::{BlackboxArgs, Mode, api, process, runtime as deployment};
use anyhow::{Result, bail};
use serde_json::Value;
use std::{path::Path, process::Command};

struct EngineProbe {
  path: String,
  expected: Fingerprint,
}

pub(super) struct HostedObjects {
  probes: Vec<EngineProbe>,
}

impl HostedObjects {
  pub fn seed(root: &Path, args: &BlackboxArgs, scenario: &api::Scenario) -> Result<Self> {
    let repository = format!(
      "projects/{}/repositories/blackbox-smoke",
      scenario.project_id
    );
    let mut reference = format!("bb-{}", scenario.suffix);
    let mut probes = Vec::new();
    for depth in 0..4 {
      let path = format!("/v2/{repository}/manifests/{reference}");
      let bytes = read(root, args, None, &path)?;
      let expected = probes::bytes_fingerprint(&bytes);
      if depth != 0 {
        probes::ensure_digest(&expected, &reference)?;
      }
      let digest = format!("sha256:{}", expected.sha256);
      probes.push(EngineProbe {
        path: format!("/v2/{repository}/manifests/{digest}"),
        expected,
      });
      let document: Value = serde_json::from_slice(&bytes)?;
      if let Some(next) = probes::child_manifest(&document)? {
        reference = next;
        continue;
      }
      for digest in probes::blob_digests(&document)? {
        let path = format!("/v2/{repository}/blobs/{digest}");
        let bytes = read(root, args, None, &path)?;
        let expected = probes::bytes_fingerprint(&bytes);
        probes::ensure_digest(&expected, &digest)?;
        probes.push(EngineProbe { path, expected });
      }
      return Ok(Self { probes });
    }
    bail!("hosted OCI index nesting exceeded backup probe limit")
  }

  pub fn verify(&self, root: &Path, args: &BlackboxArgs, restored: &Restored) -> Result<()> {
    for probe in &self.probes {
      let bytes = read(root, args, Some(restored), &probe.path)?;
      if probes::bytes_fingerprint(&bytes) != probe.expected {
        bail!("restored internal hosted OCI bytes/checksum differ");
      }
    }
    Ok(())
  }

  pub fn count(&self) -> usize {
    self.probes.len()
  }
}

fn read(
  root: &Path,
  args: &BlackboxArgs,
  restored: Option<&Restored>,
  path: &str,
) -> Result<Vec<u8>> {
  let mut command = Command::new("docker");
  if let Some(restored) = restored {
    command
      .args(["compose", "--project-name", &restored.project, "--file"])
      .arg(&restored.compose);
  } else {
    deployment::configure_compose(&mut command, root, args)?;
  }
  command
    .args([
      "exec",
      "--no-TTY",
      if args.mode == Mode::Standalone {
        "gity"
      } else {
        "registry"
      },
      "curl",
      "--fail",
      "--silent",
      "--show-error",
      "--max-time",
      "30",
      "--max-filesize",
      "8388608",
      "--header",
    ])
    .arg(format!("Accept: {}", probes::MANIFEST_ACCEPT))
    .args([
      "--header",
      "x-angos-no-redirect: true",
      "--header",
      "Accept-Encoding: identity",
    ])
    .arg(format!("http://127.0.0.1:18000{path}"));
  Ok(process::run_output(&mut command, "read internal hosted OCI backup object")?.stdout)
}
