use crate::ArtifactValidationError;
use flate2::read::GzDecoder;
use serde_json::Value;
use std::io::Read;
use tar::Archive;

pub fn npm_manifest(tarball: &[u8]) -> Result<Value, ArtifactValidationError> {
  let decoder = GzDecoder::new(tarball);
  let mut archive = Archive::new(decoder);
  for entry in archive
    .entries()
    .map_err(|_| ArtifactValidationError::new("invalid npm tarball"))?
  {
    let entry = entry.map_err(|_| ArtifactValidationError::new("invalid npm tarball"))?;
    let path = entry
      .path()
      .map_err(|_| ArtifactValidationError::new("invalid npm tarball path"))?;
    if path == std::path::Path::new("package/package.json") {
      let mut bytes = Vec::new();
      entry
        .take(1024 * 1024)
        .read_to_end(&mut bytes)
        .map_err(|_| ArtifactValidationError::new("invalid npm package manifest"))?;
      return serde_json::from_slice(&bytes)
        .map_err(|_| ArtifactValidationError::new("invalid npm package manifest"));
    }
  }
  Err(ArtifactValidationError::new(
    "npm tarball has no package/package.json",
  ))
}

pub fn sha256_integrity(hex_digest: &str) -> Result<String, ArtifactValidationError> {
  let bytes = hex::decode(hex_digest)
    .map_err(|_| ArtifactValidationError::new("invalid package checksum"))?;
  if bytes.len() != 32 {
    return Err(ArtifactValidationError::new("invalid package checksum"));
  }
  use base64::Engine;
  Ok(format!(
    "sha256-{}",
    base64::engine::general_purpose::STANDARD.encode(bytes)
  ))
}

#[cfg(test)]
#[path = "npm_tests.rs"]
mod tests;
