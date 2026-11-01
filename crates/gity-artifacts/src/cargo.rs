use crate::ArtifactValidationError;
use crate::PackageFormat;
use flate2::read::GzDecoder;
use serde_json::{Map, Value, json};
use std::io::Read;
use tar::Archive;

#[derive(Clone, Debug, PartialEq)]
pub struct CargoMetadata {
  pub name: String,
  pub version: String,
  pub dependencies: Vec<Value>,
  pub features: Map<String, Value>,
  pub links: Option<String>,
  pub rust_version: Option<String>,
}

pub fn cargo_index_path(name: &str) -> Result<String, ArtifactValidationError> {
  let name = PackageFormat::Cargo.normalize_name(name)?;
  match name.len() {
    0 => Err(ArtifactValidationError::new("Cargo crate name is empty")),
    1 => Ok(format!("1/{name}")),
    2 => Ok(format!("2/{name}")),
    3 => Ok(format!("3/{}/{name}", &name[..1])),
    _ => Ok(format!("{}/{}/{name}", &name[..2], &name[2..4])),
  }
}

pub fn cargo_metadata(crate_archive: &[u8]) -> Result<CargoMetadata, ArtifactValidationError> {
  let mut archive = Archive::new(GzDecoder::new(crate_archive));
  for entry in archive
    .entries()
    .map_err(|_| ArtifactValidationError::new("invalid Cargo crate archive"))?
  {
    let entry = entry.map_err(|_| ArtifactValidationError::new("invalid Cargo crate archive"))?;
    let path = entry
      .path()
      .map_err(|_| ArtifactValidationError::new("invalid Cargo crate archive path"))?;
    if path.file_name().is_some_and(|name| name == "Cargo.toml") {
      let mut content = String::new();
      entry
        .take(1024 * 1024)
        .read_to_string(&mut content)
        .map_err(|_| ArtifactValidationError::new("invalid Cargo manifest"))?;
      return parse_manifest(&content);
    }
  }
  Err(ArtifactValidationError::new(
    "Cargo crate has no Cargo.toml",
  ))
}

fn parse_manifest(content: &str) -> Result<CargoMetadata, ArtifactValidationError> {
  let document = content
    .parse::<toml::Value>()
    .map_err(|_| ArtifactValidationError::new("invalid Cargo manifest"))?;
  let package = document
    .get("package")
    .and_then(toml::Value::as_table)
    .ok_or_else(|| ArtifactValidationError::new("Cargo manifest package table is missing"))?;
  let name = package
    .get("name")
    .and_then(toml::Value::as_str)
    .ok_or_else(|| ArtifactValidationError::new("Cargo package name is missing"))?
    .to_owned();
  let version = package
    .get("version")
    .and_then(toml::Value::as_str)
    .ok_or_else(|| ArtifactValidationError::new("Cargo package version is missing"))?
    .to_owned();
  let dependencies = collect_dependencies(&document);
  let features = document
    .get("features")
    .and_then(toml::Value::as_table)
    .map(|features| {
      features
        .iter()
        .map(|(name, values)| (name.clone(), toml_to_json(values)))
        .collect()
    })
    .unwrap_or_default();
  Ok(CargoMetadata {
    name,
    version,
    dependencies,
    features,
    links: package
      .get("links")
      .and_then(toml::Value::as_str)
      .map(str::to_owned),
    rust_version: package
      .get("rust-version")
      .and_then(toml::Value::as_str)
      .map(str::to_owned),
  })
}

fn collect_dependencies(document: &toml::Value) -> Vec<Value> {
  let mut dependencies = Vec::new();
  for kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
    if let Some(table) = document.get(kind).and_then(toml::Value::as_table) {
      collect_dependency_table(table, kind, None, &mut dependencies);
    }
  }
  if let Some(targets) = document.get("target").and_then(toml::Value::as_table) {
    for (target, target_table) in targets {
      for kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = target_table.get(kind).and_then(toml::Value::as_table) {
          collect_dependency_table(table, kind, Some(target), &mut dependencies);
        }
      }
    }
  }
  dependencies
}

fn collect_dependency_table(
  table: &toml::map::Map<String, toml::Value>,
  kind: &str,
  target: Option<&str>,
  output: &mut Vec<Value>,
) {
  for (alias, value) in table {
    let details = value.as_table();
    let name = details
      .and_then(|details| details.get("package"))
      .and_then(toml::Value::as_str)
      .unwrap_or(alias);
    let requirement = details
      .and_then(|details| details.get("version"))
      .and_then(toml::Value::as_str)
      .or_else(|| value.as_str())
      .unwrap_or("*");
    let mut dependency = json!({
        "name":name,
        "req":requirement,
        "features":details.and_then(|details| details.get("features"))
            .map(toml_to_json).unwrap_or(json!([])),
        "optional":details.and_then(|details| details.get("optional"))
            .and_then(toml::Value::as_bool).unwrap_or(false),
        "default_features":details.and_then(|details| details.get("default-features"))
            .and_then(toml::Value::as_bool).unwrap_or(true),
        "target":target,
        "kind":match kind { "dev-dependencies"=>"dev", "build-dependencies"=>"build", _=>"normal" },
    });
    if name != alias {
      dependency["package"] = json!(alias);
    }
    if let Some(registry) = details
      .and_then(|details| details.get("registry"))
      .and_then(toml::Value::as_str)
    {
      dependency["registry"] = json!(registry);
    }
    output.push(dependency);
  }
}

fn toml_to_json(value: &toml::Value) -> Value {
  match value {
    toml::Value::String(value) => json!(value),
    toml::Value::Integer(value) => json!(value),
    toml::Value::Float(value) => json!(value),
    toml::Value::Boolean(value) => json!(value),
    toml::Value::Datetime(value) => json!(value.to_string()),
    toml::Value::Array(values) => Value::Array(values.iter().map(toml_to_json).collect()),
    toml::Value::Table(table) => Value::Object(
      table
        .iter()
        .map(|(key, value)| (key.clone(), toml_to_json(value)))
        .collect(),
    ),
  }
}

#[cfg(test)]
#[path = "cargo_tests.rs"]
mod tests;
