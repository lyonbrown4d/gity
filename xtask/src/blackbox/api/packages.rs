use super::super::{Reporter, RunConfig};
use super::{require_denied, require_status};
use anyhow::{Context, Result, bail};
use reqwest::{StatusCode, blocking::Client};
use sha2::{Digest, Sha256};

pub(super) fn run(
  client: &Client,
  config: &RunConfig,
  project_id: &str,
  owner_token: &str,
  outsider_token: &str,
  suffix: &str,
  reporter: &mut Reporter,
) -> Result<()> {
  let version = format!("1.0.{:x}", suffix.as_bytes()[0]);
  let path = format!(
    "{}/projects/{project_id}/packages/generic/blackbox/{version}/payload.bin",
    config.api_url
  );
  let payload = fixture_bytes(suffix);
  let digest = hex::encode(Sha256::digest(&payload));

  let first = client
    .put(&path)
    .bearer_auth(owner_token)
    .header("content-type", "application/octet-stream")
    .body(payload.clone())
    .send()
    .context("upload Generic package fixture")?;
  require_status(first.status(), StatusCode::OK, "Generic package upload")?;
  let file_record: serde_json::Value = first
    .json()
    .context("decode Generic package upload response")?;
  if super::data(&file_record)["sha256"].as_str() != Some(digest.as_str()) {
    bail!("Generic package upload returned an unexpected SHA-256");
  }

  let repeated = client
    .put(&path)
    .bearer_auth(owner_token)
    .header("content-type", "application/octet-stream")
    .body(payload.clone())
    .send()
    .context("retry identical Generic package upload")?;
  require_status(
    repeated.status(),
    StatusCode::OK,
    "idempotent Generic package retry",
  )?;

  let replacement = client
    .put(&path)
    .bearer_auth(owner_token)
    .header("content-type", "application/octet-stream")
    .body(b"replacement bytes must conflict".to_vec())
    .send()
    .context("attempt conflicting Generic package upload")?;
  require_status(
    replacement.status(),
    StatusCode::CONFLICT,
    "conflicting Generic package replacement",
  )?;

  let range_start = 1024_usize;
  let range_end = 4095_usize;
  let range = client
    .get(&path)
    .bearer_auth(owner_token)
    .header("range", format!("bytes={range_start}-{range_end}"))
    .send()
    .context("download Generic package byte range")?;
  require_status(
    range.status(),
    StatusCode::PARTIAL_CONTENT,
    "Generic Range response",
  )?;
  let content_range = range
    .headers()
    .get("content-range")
    .and_then(|value| value.to_str().ok())
    .unwrap_or_default();
  if content_range != format!("bytes {range_start}-{range_end}/{}", payload.len()) {
    bail!("Generic Range response returned an unexpected Content-Range");
  }
  let range_bytes = range.bytes().context("read Generic Range bytes")?;
  if range_bytes.as_ref() != &payload[range_start..=range_end] {
    bail!("Generic Range response bytes did not match the uploaded object");
  }

  let download = client
    .get(&path)
    .bearer_auth(owner_token)
    .send()
    .context("download Generic package fixture")?;
  require_status(
    download.status(),
    StatusCode::OK,
    "Generic package download",
  )?;
  let checksum = download
    .headers()
    .get("x-checksum-sha256")
    .and_then(|value| value.to_str().ok())
    .unwrap_or_default()
    .to_owned();
  let downloaded = download.bytes().context("read Generic package bytes")?;
  if downloaded.as_ref() != payload || checksum != digest {
    bail!("Generic package download or SHA-256 did not match the upload");
  }

  let outsider = client
    .get(&path)
    .bearer_auth(outsider_token)
    .send()
    .context("check cross-tenant package access")?;
  require_denied(outsider.status(), "cross-tenant package read")?;
  reporter.pass(
    "generic-upload-download-range-sha-conflict-tenant",
    format!(
      "{} bytes; idempotent retry; replacement conflict; exact Range and SHA-256; outsider denied",
      payload.len()
    ),
  );
  Ok(())
}

fn fixture_bytes(seed: &str) -> Vec<u8> {
  let mut data = Vec::with_capacity(12 * 1024 * 1024);
  let mut index = 0_u64;
  while data.len() < 12 * 1024 * 1024 {
    let block = Sha256::digest(format!("{seed}:{index}"));
    data.extend_from_slice(&block);
    index += 1;
  }
  data.truncate(12 * 1024 * 1024);
  data
}
