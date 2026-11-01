use crate::ArtifactValidationError;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::Cursor;
use zip::ZipArchive;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NugetMetadata {
  pub id: String,
  pub version: String,
  pub nuspec: String,
  pub dependency_groups: Vec<NugetDependencyGroup>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NugetDependencyGroup {
  pub target_framework: Option<String>,
  pub dependencies: Vec<NugetDependency>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NugetDependency {
  pub id: String,
  pub range: String,
}

pub fn nuget_metadata(package: &[u8]) -> Result<NugetMetadata, ArtifactValidationError> {
  let mut archive = ZipArchive::new(Cursor::new(package))
    .map_err(|_| ArtifactValidationError::new("invalid NuGet package archive"))?;
  let name = (0..archive.len())
    .find_map(|index| {
      let entry = archive.by_index(index).ok()?;
      entry
        .name()
        .ends_with(".nuspec")
        .then(|| entry.name().to_owned())
    })
    .ok_or_else(|| ArtifactValidationError::new("NuGet package is missing its nuspec"))?;
  let mut nuspec = String::new();
  archive
    .by_name(&name)
    .map_err(|_| ArtifactValidationError::new("invalid NuGet package manifest"))?
    .read_to_string(&mut nuspec)
    .map_err(|_| ArtifactValidationError::new("invalid NuGet package manifest"))?;
  let (id, version) = parse_identity(&nuspec)?;
  let dependency_groups = parse_dependencies(&nuspec)?;
  Ok(NugetMetadata {
    id,
    version,
    nuspec,
    dependency_groups,
  })
}

fn parse_identity(nuspec: &str) -> Result<(String, String), ArtifactValidationError> {
  let mut reader = Reader::from_str(nuspec);
  reader.config_mut().trim_text(true);
  let mut current = None;
  let mut id = None;
  let mut version = None;
  loop {
    match reader.read_event() {
      Ok(Event::Start(element)) => match element.local_name().as_ref() {
        b"id" => current = Some("id"),
        b"version" => current = Some("version"),
        _ => {}
      },
      Ok(Event::Text(text)) => {
        let value = text
          .decode()
          .map_err(|_| ArtifactValidationError::new("invalid NuGet package manifest"))?
          .trim()
          .to_owned();
        match current.take() {
          Some("id") if id.is_none() => id = Some(value),
          Some("version") if version.is_none() => version = Some(value),
          _ => {}
        }
      }
      Ok(Event::End(_)) => current = None,
      Ok(Event::Eof) => break,
      Ok(_) => {}
      Err(_) => {
        return Err(ArtifactValidationError::new(
          "invalid NuGet package manifest",
        ));
      }
    }
  }
  let id = id
    .filter(|value| !value.is_empty())
    .ok_or_else(|| ArtifactValidationError::new("NuGet package id is missing"))?;
  let version = version
    .filter(|value| !value.is_empty())
    .ok_or_else(|| ArtifactValidationError::new("NuGet package version is missing"))?;
  Ok((id, version))
}

fn parse_dependencies(nuspec: &str) -> Result<Vec<NugetDependencyGroup>, ArtifactValidationError> {
  let mut reader = Reader::from_str(nuspec);
  let mut groups = Vec::<NugetDependencyGroup>::new();
  let mut current_group = None;
  loop {
    match reader.read_event() {
      Ok(Event::Start(element)) | Ok(Event::Empty(element))
        if element.local_name().as_ref() == b"group" =>
      {
        let target_framework = element.attributes().flatten().find_map(|attribute| {
          (attribute.key.local_name().as_ref() == b"targetFramework")
            .then(|| {
              attribute
                .decode_and_unescape_value(reader.decoder())
                .ok()
                .map(|value| value.into_owned())
            })
            .flatten()
        });
        groups.push(NugetDependencyGroup {
          target_framework,
          dependencies: Vec::new(),
        });
        current_group = Some(groups.len() - 1);
      }
      Ok(Event::Start(element)) | Ok(Event::Empty(element))
        if element.local_name().as_ref() == b"dependency" =>
      {
        let mut id = None;
        let mut range = None;
        for attribute in element.attributes().flatten() {
          let value = attribute
            .decode_and_unescape_value(reader.decoder())
            .map_err(|_| ArtifactValidationError::new("invalid NuGet dependency metadata"))?
            .into_owned();
          match attribute.key.local_name().as_ref() {
            b"id" => id = Some(value),
            b"version" => range = Some(value),
            _ => {}
          }
        }
        let id =
          id.ok_or_else(|| ArtifactValidationError::new("NuGet dependency id is missing"))?;
        let index = current_group.unwrap_or_else(|| {
          groups.push(NugetDependencyGroup {
            target_framework: None,
            dependencies: Vec::new(),
          });
          groups.len() - 1
        });
        groups[index].dependencies.push(NugetDependency {
          id,
          range: range.unwrap_or_default(),
        });
      }
      Ok(Event::End(element)) if element.local_name().as_ref() == b"group" => {
        current_group = None;
      }
      Ok(Event::Eof) => break,
      Ok(_) => {}
      Err(_) => {
        return Err(ArtifactValidationError::new(
          "invalid NuGet dependency metadata",
        ));
      }
    }
  }
  Ok(groups)
}

use std::io::Read;

#[cfg(test)]
#[path = "nuget_tests.rs"]
mod tests;
