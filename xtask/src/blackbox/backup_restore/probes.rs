//! Public authenticated reads are separate from administrative hosted-engine reads.
use crate::blackbox::{RunConfig, api};
use anyhow::{Context, Result, bail};
use reqwest::{
  StatusCode,
  blocking::{Client, Response},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::Read;

pub(super) const MANIFEST_ACCEPT: &str = "application/vnd.oci.image.index.v1+json, application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.docker.distribution.manifest.v2+json";

#[derive(Clone, PartialEq, Eq)]
pub(super) struct Fingerprint {
  pub size: u64,
  pub sha256: String,
}

pub(super) struct PublicProbe {
  name: &'static str,
  url: String,
  expected: Fingerprint,
}

pub(super) struct PublicObjects {
  probes: Vec<PublicProbe>,
}

impl PublicObjects {
  pub fn seed(client: &Client, config: &RunConfig, scenario: &api::Scenario) -> Result<Self> {
    let version = format!("1.0.{:x}", scenario.suffix.as_bytes()[0]);
    let generic = format!(
      "{}/projects/{}/packages/generic/blackbox/{version}/payload.bin",
      config.api_url, scenario.project_id
    );
    let expected = fingerprint(response(client, &generic, &scenario.owner_token)?)?;
    if expected.size != 12 * 1024 * 1024 {
      bail!("existing Generic backup fixture size differs");
    }
    let mut probes = vec![PublicProbe {
      name: "Generic package",
      url: generic,
      expected,
    }];
    let repository = format!("{}/v2/{}", config.oci_url, scenario.project_path);
    let mut reference = "3.22".to_owned();
    for depth in 0..4 {
      let bytes = manifest(
        client,
        &format!("{repository}/manifests/{reference}"),
        &scenario.owner_token,
      )?;
      let expected = bytes_fingerprint(&bytes);
      let digest = format!("sha256:{}", expected.sha256);
      if depth != 0 && reference != digest {
        bail!("mirror child manifest digest mismatch");
      }
      // Freeze the observed immutable digest; do not mutate the source policy or
      // let a later upstream tag update change the restore comparison.
      probes.push(PublicProbe {
        name: "mirror manifest",
        url: format!("{repository}/manifests/{digest}"),
        expected,
      });
      let document: Value = serde_json::from_slice(&bytes)?;
      if let Some(next) = child_manifest(&document)? {
        reference = next;
        continue;
      }
      let digests = blob_digests(&document)?;
      for digest in digests {
        let url = format!("{repository}/blobs/{digest}");
        let expected = fingerprint(response(client, &url, &scenario.owner_token)?)?;
        ensure_digest(&expected, &digest)?;
        probes.push(PublicProbe {
          name: "mirror blob",
          url,
          expected,
        });
      }
      return Ok(Self { probes });
    }
    bail!("mirror index nesting exceeded backup probe limit")
  }

  pub fn verify(&self, client: &Client, scenario: &api::Scenario) -> Result<()> {
    for probe in &self.probes {
      let actual = fingerprint(response(client, &probe.url, &scenario.owner_token)?)?;
      if actual != probe.expected {
        bail!("restored {} bytes/checksum differ", probe.name);
      }
      api::require_denied(
        client
          .get(&probe.url)
          .header("accept", MANIFEST_ACCEPT)
          .bearer_auth(&scenario.outsider_token)
          .send()?
          .status(),
        "restored package/mirror outsider read",
      )?;
    }
    Ok(())
  }

  pub fn count(&self) -> usize {
    self.probes.len()
  }
}

fn response(client: &Client, url: &str, token: &str) -> Result<Response> {
  let response = client
    .get(url)
    .header("accept", MANIFEST_ACCEPT)
    .header("accept-encoding", "identity")
    .bearer_auth(token)
    .send()?;
  api::require_status(
    response.status(),
    StatusCode::OK,
    "backup public object read",
  )?;
  Ok(response)
}

fn manifest(client: &Client, url: &str, token: &str) -> Result<Vec<u8>> {
  let response = response(client, url, token)?;
  let mut bytes = Vec::new();
  response.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
  if bytes.len() > 4 * 1024 * 1024 {
    bail!("manifest exceeds backup probe limit");
  }
  Ok(bytes)
}

pub(super) fn fingerprint(mut reader: impl Read) -> Result<Fingerprint> {
  let mut digest = Sha256::new();
  let mut size = 0;
  let mut buffer = [0; 64 * 1024];
  loop {
    let count = reader.read(&mut buffer)?;
    if count == 0 {
      break;
    }
    digest.update(&buffer[..count]);
    size += count as u64;
  }
  Ok(Fingerprint {
    size,
    sha256: hex::encode(digest.finalize()),
  })
}

pub(super) fn bytes_fingerprint(bytes: &[u8]) -> Fingerprint {
  Fingerprint {
    size: bytes.len() as u64,
    sha256: hex::encode(Sha256::digest(bytes)),
  }
}

pub(super) fn ensure_digest(value: &Fingerprint, digest: &str) -> Result<()> {
  if digest != format!("sha256:{}", value.sha256) {
    bail!("OCI object does not match its content digest");
  }
  Ok(())
}

pub(super) fn child_manifest(document: &Value) -> Result<Option<String>> {
  let Some(children) = document["manifests"].as_array() else {
    return Ok(None);
  };
  let arch = match std::env::consts::ARCH {
    "x86_64" => "amd64",
    "aarch64" => "arm64",
    other => other,
  };
  let child = children
    .iter()
    .find(|child| child["platform"]["os"] == "linux" && child["platform"]["architecture"] == arch)
    .context("manifest index lacks the current Linux architecture")?;
  Ok(Some(content_digest(&child["digest"])?))
}

pub(super) fn blob_digests(document: &Value) -> Result<Vec<String>> {
  let config = content_digest(&document["config"]["digest"])?;
  let layers = document["layers"]
    .as_array()
    .context("manifest layers missing")?;
  let mut digests = vec![config];
  for layer in layers {
    digests.push(content_digest(&layer["digest"])?);
  }
  Ok(digests)
}

fn content_digest(value: &Value) -> Result<String> {
  let value = value.as_str().context("OCI descriptor digest missing")?;
  let hash = value
    .strip_prefix("sha256:")
    .context("OCI digest must use SHA256")?;
  if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
    bail!("invalid OCI descriptor digest");
  }
  Ok(value.to_owned())
}
