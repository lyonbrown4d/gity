use crate::ArtifactValidationError;
use flate2::read::GzDecoder;
use std::io::{Cursor, Read};
use tar::Archive;
use zip::ZipArchive;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PythonMetadata {
  pub name: String,
  pub version: String,
  pub requires_python: Option<String>,
  pub requires_dist: Vec<String>,
}

pub fn python_metadata(
  filename: &str,
  package: &[u8],
) -> Result<PythonMetadata, ArtifactValidationError> {
  let metadata = if filename.ends_with(".whl") {
    wheel_metadata(package)?
  } else if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
    sdist_metadata(package)?
  } else {
    return Err(ArtifactValidationError::new(
      "unsupported PyPI artifact format",
    ));
  };
  let headers = parse_headers(&metadata)?;
  let name = first_header(&headers, "name")
    .ok_or_else(|| ArtifactValidationError::new("Python package name is missing"))?;
  let version = first_header(&headers, "version")
    .ok_or_else(|| ArtifactValidationError::new("Python package version is missing"))?;
  pep440_rs::Version::from_str(&version)
    .map_err(|_| ArtifactValidationError::new("Python package version is invalid"))?;
  let requires_python = first_header(&headers, "requires-python");
  let requires_dist = headers.get("requires-dist").cloned().unwrap_or_default();
  Ok(PythonMetadata {
    name: normalize_name(&name)?,
    version,
    requires_python,
    requires_dist,
  })
}

fn wheel_metadata(bytes: &[u8]) -> Result<String, ArtifactValidationError> {
  let mut archive = ZipArchive::new(Cursor::new(bytes))
    .map_err(|_| ArtifactValidationError::new("invalid Python wheel archive"))?;
  for index in 0..archive.len() {
    let entry = archive
      .by_index(index)
      .map_err(|_| ArtifactValidationError::new("invalid Python wheel archive"))?;
    if entry.name().ends_with(".dist-info/METADATA") {
      let mut output = String::new();
      entry
        .take(1024 * 1024)
        .read_to_string(&mut output)
        .map_err(|_| ArtifactValidationError::new("invalid wheel metadata"))?;
      return Ok(output);
    }
  }
  Err(ArtifactValidationError::new(
    "wheel is missing dist-info/METADATA",
  ))
}

fn sdist_metadata(bytes: &[u8]) -> Result<String, ArtifactValidationError> {
  let mut archive = Archive::new(GzDecoder::new(bytes));
  for entry in archive
    .entries()
    .map_err(|_| ArtifactValidationError::new("invalid Python source archive"))?
  {
    let entry = entry.map_err(|_| ArtifactValidationError::new("invalid Python source archive"))?;
    if entry
      .path()
      .ok()
      .is_some_and(|path| path.file_name().is_some_and(|name| name == "PKG-INFO"))
    {
      let mut output = String::new();
      entry
        .take(1024 * 1024)
        .read_to_string(&mut output)
        .map_err(|_| ArtifactValidationError::new("invalid Python source metadata"))?;
      return Ok(output);
    }
  }
  Err(ArtifactValidationError::new(
    "source archive is missing PKG-INFO",
  ))
}

fn parse_headers(
  content: &str,
) -> Result<std::collections::BTreeMap<String, Vec<String>>, ArtifactValidationError> {
  let mut headers = std::collections::BTreeMap::<String, Vec<String>>::new();
  let mut current = None::<String>;
  for line in content.lines() {
    if line.is_empty() {
      break;
    }
    if line.starts_with([' ', '\t']) {
      if let Some(key) = &current
        && let Some(values) = headers.get_mut(key)
        && let Some(value) = values.last_mut()
      {
        value.push(' ');
        value.push_str(line.trim());
      }
      continue;
    }
    let (key, value) = line
      .split_once(':')
      .ok_or_else(|| ArtifactValidationError::new("invalid package metadata header"))?;
    let key = key.trim().to_ascii_lowercase();
    headers
      .entry(key.clone())
      .or_default()
      .push(value.trim().to_owned());
    current = Some(key);
  }
  Ok(headers)
}

fn first_header(
  headers: &std::collections::BTreeMap<String, Vec<String>>,
  key: &str,
) -> Option<String> {
  headers.get(key).and_then(|values| values.first()).cloned()
}

fn normalize_name(value: &str) -> Result<String, ArtifactValidationError> {
  gity_artifacts_normalize_python_name(value)
}

fn gity_artifacts_normalize_python_name(value: &str) -> Result<String, ArtifactValidationError> {
  use crate::PackageFormat;
  PackageFormat::Pypi.normalize_name(value)
}

use std::str::FromStr;

#[cfg(test)]
#[path = "python_tests.rs"]
mod tests;
