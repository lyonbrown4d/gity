use crate::ArtifactValidationError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageFormat {
  Generic,
  Cargo,
  Npm,
  Maven,
  Pypi,
  Nuget,
  Go,
}

impl PackageFormat {
  pub fn parse(value: &str) -> Option<Self> {
    match value {
      "generic" => Some(Self::Generic),
      "cargo" => Some(Self::Cargo),
      "npm" => Some(Self::Npm),
      "maven" => Some(Self::Maven),
      "pypi" => Some(Self::Pypi),
      "nuget" => Some(Self::Nuget),
      "go" => Some(Self::Go),
      _ => None,
    }
  }

  pub fn as_str(self) -> &'static str {
    match self {
      Self::Generic => "generic",
      Self::Cargo => "cargo",
      Self::Npm => "npm",
      Self::Maven => "maven",
      Self::Pypi => "pypi",
      Self::Nuget => "nuget",
      Self::Go => "go",
    }
  }

  pub fn validate_version(self, value: &str) -> Result<(), ArtifactValidationError> {
    let valid = match self {
      Self::Cargo | Self::Npm | Self::Nuget => semver::Version::parse(value).is_ok(),
      Self::Pypi => pep440_rs::Version::from_str(value).is_ok(),
      Self::Go => semver::Version::parse(value.strip_prefix('v').unwrap_or(value)).is_ok(),
      Self::Generic | Self::Maven => !value.trim().is_empty(),
    };
    if valid {
      Ok(())
    } else {
      Err(ArtifactValidationError::new("package version is invalid"))
    }
  }

  pub fn normalize_name(self, name: &str) -> Result<String, ArtifactValidationError> {
    let name = name.trim();
    if name.is_empty() || name.contains('\0') || name.contains('\\') {
      return Err(ArtifactValidationError::new("package name is invalid"));
    }
    if matches!(self, Self::Cargo | Self::Nuget | Self::Pypi)
      && !name
        .bytes()
        .all(|value| value.is_ascii_alphanumeric() || matches!(value, b'-' | b'_' | b'.'))
    {
      return Err(ArtifactValidationError::new("package name is invalid"));
    }
    Ok(match self {
      Self::Cargo => name.to_ascii_lowercase().replace('_', "-"),
      Self::Nuget => name.to_ascii_lowercase(),
      Self::Pypi => {
        let mut normalized = String::new();
        let mut separator = false;
        for ch in name.chars() {
          if matches!(ch, '-' | '_' | '.') {
            if !separator {
              normalized.push('-');
            }
            separator = true;
          } else {
            normalized.extend(ch.to_lowercase());
            separator = false;
          }
        }
        normalized
      }
      _ => name.to_owned(),
    })
  }
}

use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactCoordinate {
  pub project_id: i64,
  pub format: PackageFormat,
  pub name: String,
  pub version: String,
  pub path: String,
}

impl ArtifactCoordinate {
  pub fn new(
    project_id: i64,
    format: PackageFormat,
    name: impl Into<String>,
    version: impl Into<String>,
    path: impl Into<String>,
  ) -> Result<Self, ArtifactValidationError> {
    let name = format.normalize_name(&name.into())?;
    let version = version.into();
    let path = path.into();
    if project_id <= 0 || name.trim().is_empty() || version.trim().is_empty() {
      return Err(ArtifactValidationError::new(
        "package coordinate is incomplete",
      ));
    }
    format.validate_version(&version)?;
    if path.is_empty()
      || path.starts_with('/')
      || path.contains('\\')
      || path.contains('\0')
      || path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
      return Err(ArtifactValidationError::new("invalid package file path"));
    }
    Ok(Self {
      project_id,
      format,
      name,
      version,
      path,
    })
  }
}

#[cfg(test)]
#[path = "domain_tests.rs"]
mod tests;
