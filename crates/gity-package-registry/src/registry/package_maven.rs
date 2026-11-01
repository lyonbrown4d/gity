use super::packages::named;
use super::*;
use axum::{
  http::{HeaderMap, header},
  response::{IntoResponse, Response},
};
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256};

pub(super) async fn metadata(
  s: &AppState,
  h: &HeaderMap,
  project_id: i64,
  path: &str,
) -> Result<Response, AppError> {
  authorize(s, h, project_id, PackageAction::Read).await?;
  Ok(xml_response(metadata_xml(s, project_id, path).await?))
}

pub(super) async fn metadata_checksum(
  s: &AppState,
  h: &HeaderMap,
  project_id: i64,
  path: &str,
) -> Result<Response, AppError> {
  authorize(s, h, project_id, PackageAction::Read).await?;
  let (metadata_path, algorithm) = metadata_checksum_path(path)
    .ok_or_else(|| AppError::bad_request("invalid Maven metadata checksum path"))?;
  let xml = metadata_xml(s, project_id, metadata_path).await?;
  let checksum = if algorithm == ".sha1" {
    hex::encode(Sha1::digest(xml.as_bytes()))
  } else if algorithm == ".md5" {
    hex::encode(Md5::digest(xml.as_bytes()))
  } else {
    hex::encode(Sha256::digest(xml.as_bytes()))
  };
  Ok(
    (
      [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
      format!("{checksum}\n"),
    )
      .into_response(),
  )
}

pub(super) fn metadata_checksum_path(path: &str) -> Option<(&str, &str)> {
  [".sha1", ".md5", ".sha256"].iter().find_map(|suffix| {
    path
      .strip_suffix(suffix)
      .filter(|base| base.ends_with("/maven-metadata.xml"))
      .map(|base| (base, *suffix))
  })
}

pub(super) fn validate_metadata_checksum(bytes: &[u8], algorithm: &str) -> Result<(), AppError> {
  let expected = match algorithm {
    ".sha1" => 40,
    ".md5" => 32,
    ".sha256" => 64,
    _ => return Err(AppError::bad_request("unsupported Maven metadata checksum")),
  };
  let checksum = std::str::from_utf8(bytes)
    .map_err(|_| AppError::bad_request("invalid Maven metadata checksum"))?
    .trim();
  if checksum.len() != expected || !checksum.bytes().all(|value| value.is_ascii_hexdigit()) {
    return Err(AppError::bad_request("invalid Maven metadata checksum"));
  }
  Ok(())
}

#[path = "package_maven_metadata.rs"]
mod metadata;
pub(super) use metadata::validate_metadata_document;
use metadata::*;

#[cfg(test)]
#[path = "package_maven_tests.rs"]
mod tests;
