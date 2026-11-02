use super::{
  BackupArgs, docker,
  model::{self, Bind, Image, Manifest, Volume},
};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::{
  collections::{BTreeMap, BTreeSet},
  path::{Path, PathBuf},
};

pub struct Sources {
  volumes: Vec<Volume>,
  binds: Vec<(String, PathBuf, bool)>,
  images: Vec<Image>,
}

pub fn apply_container_identity(config: &mut Value, containers: &[Value]) -> Result<()> {
  for container in containers {
    let service = container["Config"]["Labels"]["com.docker.compose.service"]
      .as_str()
      .context("container has no Compose service identity")?;
    let Some(target) = config["services"].get_mut(service) else {
      bail!("project contains a service absent from supplied Compose file");
    };
    let env = container["Config"]["Env"]
      .as_array()
      .context("container environment missing")?;
    let mut values = serde_json::Map::new();
    for entry in env {
      if let Some((key, value)) = entry.as_str().and_then(|s| s.split_once('=')) {
        values.insert(key.into(), Value::String(value.into()));
      }
    }
    target["environment"] = Value::Object(values);
  }
  Ok(())
}

pub fn sources(config: &Value, project: &str, containers: &[Value]) -> Result<Sources> {
  for key in ["configs", "secrets"] {
    if config[key]
      .as_object()
      .is_some_and(|values| !values.is_empty())
    {
      bail!("Compose configs/secrets objects are unsupported; use explicit protected bind sources");
    }
  }
  let mut volumes = Vec::new();
  let mut volume_names = BTreeSet::new();
  let existing: BTreeSet<_> = docker::names("volume", project)?.into_iter().collect();
  let definitions = config["volumes"]
    .as_object()
    .context("Compose project declares no volumes")?;
  for (key, definition) in definitions {
    model::safe_component(key)?;
    let name = definition["name"]
      .as_str()
      .context("volume has no resolved name")?;
    if !volume_names.insert(name.to_owned()) {
      bail!("volume aliases are unsupported for isolated restore");
    }
    if !name.starts_with(&format!("{project}_")) || definition["external"] == true {
      bail!("shared or external volumes cannot be snapshotted as a private project");
    }
    if docker::volume_exists(name)? && !existing.contains(name) {
      bail!("configured volume is not owned by the exact source project");
    }
    volumes.push(Volume {
      key: key.clone(),
      source_name: name.into(),
      archive: None,
    });
  }
  if existing
    .iter()
    .any(|name| !volumes.iter().any(|v| &v.source_name == name))
  {
    bail!("project has undeclared volumes; use its complete Compose configuration");
  }
  super::ownership::verify_actual_mounts(config, containers)?;
  super::ownership::verify_no_external_writers(project, &volumes)?;
  let mut binds = BTreeMap::new();
  for service in config["services"]
    .as_object()
    .context("Compose services missing")?
    .values()
  {
    for mount in service["volumes"].as_array().into_iter().flatten() {
      match mount["type"].as_str() {
        Some("bind") => {
          let original = mount["source"].as_str().context("bind has no source")?;
          let path = Path::new(original)
            .canonicalize()
            .context("resolve required bind source")?;
          let metadata = std::fs::metadata(&path)?;
          if !metadata.is_file() && !metadata.is_dir() {
            bail!("only regular file/directory bind sources can be archived");
          }
          binds.insert(original.to_owned(), (path, metadata.is_dir()));
        }
        Some("volume") => {
          let key = mount["source"]
            .as_str()
            .context("anonymous volumes are unsupported")?;
          if !definitions.contains_key(key) {
            bail!("volume mount has no declaration");
          }
        }
        _ => bail!("unsupported mount type in Compose backup"),
      }
    }
  }
  let mut images = Vec::new();
  for (service, definition) in config["services"].as_object().unwrap() {
    let reference = definition["image"]
      .as_str()
      .context("every service must have an image reference")?;
    let image_id = if let Some(container) = containers
      .iter()
      .find(|c| c["Config"]["Labels"]["com.docker.compose.service"] == service.as_str())
    {
      container["Image"]
        .as_str()
        .context("container image identity missing")?
        .into()
    } else {
      let value: Value = serde_json::from_slice(&docker::output(
        docker::command().args(["image", "inspect", reference]),
        "inspect configured image",
      )?)?;
      value[0]["Id"]
        .as_str()
        .context("configured image identity missing")?
        .to_owned()
    };
    images.push(Image {
      service: service.clone(),
      reference: reference.into(),
      image_id,
    });
  }
  Ok(Sources {
    volumes,
    binds: binds.into_iter().map(|(s, (p, d))| (s, p, d)).collect(),
    images,
  })
}

pub fn write_bundle(
  args: &BackupArgs,
  output: &Path,
  config: Value,
  mut sources: Sources,
) -> Result<Manifest> {
  let compose_path = output.join("compose.json");
  model::write_private(&compose_path, &serde_json::to_vec_pretty(&config)?)?;
  for (index, volume) in sources.volumes.iter_mut().enumerate() {
    if docker::volume_exists(&volume.source_name)? {
      volume.archive = Some(docker::archive(
        &args.helper_image,
        "volume",
        &volume.source_name,
        model::TarPolicy::volume(&config, &volume.key),
        &output.join(format!("volume-{index:04}.tar")),
      )?);
    }
  }
  let mut binds = Vec::new();
  for (index, (source, path, directory)) in sources.binds.into_iter().enumerate() {
    if output.starts_with(&path) {
      bail!("backup output is inside a captured bind source");
    }
    let archive = docker::archive(
      &args.helper_image,
      "bind",
      path.to_str().context("bind path must be UTF-8")?,
      if directory {
        model::TarPolicy::Directory
      } else {
        model::TarPolicy::FileBind
      },
      &output.join(format!("bind-{index:04}.tar")),
    )?;
    binds.push(Bind {
      id: format!("bind-{index:04}"),
      source,
      directory,
      archive,
    });
  }
  let manifest = Manifest {
    format_version: model::VERSION,
    source_project: args.project.clone(),
    created_at: uuid::Uuid::now_v7().to_string(),
    consistency: "offline-all-project-containers-stopped".into(),
    compose: model::identity(&compose_path)?,
    volumes: sources.volumes,
    binds,
    images: sources.images,
  };
  model::write_private(
    &output.join("manifest.json"),
    &serde_json::to_vec_pretty(&manifest)?,
  )?;
  // Persist directory entries as well as individual archive contents.
  std::fs::File::open(output)?.sync_all()?;
  Ok(manifest)
}
