use super::{docker, model::Volume};
use anyhow::{Context, Result, bail};
use serde_json::Value;

/// Refuse a consistent-cut claim if another project's live container writes the
/// same named data. Never stop or modify that other container.
pub fn verify_no_external_writers(project: &str, volumes: &[Volume]) -> Result<()> {
  for volume in volumes {
    let bytes = docker::output(
      docker::command().args([
        "ps",
        "--quiet",
        "--filter",
        &format!("volume={}", volume.source_name),
      ]),
      "check volume writer isolation",
    )?;
    let ids: Vec<_> = std::str::from_utf8(&bytes)?.split_whitespace().collect();
    if ids.is_empty() {
      continue;
    }
    let containers: Vec<Value> = serde_json::from_slice(&docker::output(
      docker::command().arg("inspect").args(ids),
      "inspect volume writer ownership",
    )?)?;
    for container in containers {
      if container["Config"]["Labels"]["com.docker.compose.project"] != project {
        bail!("a running container outside the exact project mounts snapshot data; backup refused");
      }
    }
  }
  Ok(())
}

/// Docker volume-create returns an existing volume on a same-name race. A fresh
/// invocation label proves it is our allocation before extracting or cleaning it.
pub fn create_volume(project: &str, key: &str, invocation: &str) -> Result<String> {
  create_volume_with(
    project,
    key,
    invocation,
    &|args, action| docker::output(docker::command().args(args), action),
    docker::volume_exists,
  )
}

fn create_volume_with<F>(
  project: &str,
  key: &str,
  invocation: &str,
  run: &F,
  exists: impl FnOnce(&str) -> Result<bool>,
) -> Result<String>
where
  F: Fn(&[&str], &str) -> Result<Vec<u8>>,
{
  let name = format!("{project}_{key}");
  super::model::safe_component(&name)?;
  if exists(&name)? {
    bail!("target volume already exists; refusing overwrite");
  }
  // Arm before create: even a failed Docker response may follow a successful
  // daemon allocation. Cleanup always obtains fresh, matching ownership proof.
  let mut pending = PendingVolume {
    name: &name,
    project,
    key,
    invocation,
    run,
    armed: true,
  };
  let result = run(
    &[
      "volume",
      "create",
      "--label",
      &format!("com.docker.compose.project={project}"),
      "--label",
      &format!("com.docker.compose.volume={key}"),
      "--label",
      &format!("io.gity.backup.restore={invocation}"),
      &name,
    ],
    "create new restore volume",
  )?;
  if String::from_utf8(result)?.trim() != name {
    bail!("unexpected allocated volume identity");
  }
  let inspected: Value = serde_json::from_slice(&run(
    &["volume", "inspect", &name],
    "verify fresh volume allocation",
  )?)?;
  if !pending.owns(&inspected) {
    bail!("target volume was concurrently created; refusing access");
  }
  pending.armed = false;
  drop(pending);
  Ok(name)
}

struct PendingVolume<'a, F: Fn(&[&str], &str) -> Result<Vec<u8>>> {
  name: &'a str,
  project: &'a str,
  key: &'a str,
  invocation: &'a str,
  run: &'a F,
  armed: bool,
}
impl<F: Fn(&[&str], &str) -> Result<Vec<u8>>> PendingVolume<'_, F> {
  fn owns(&self, inspected: &Value) -> bool {
    let volume = &inspected[0];
    volume["Name"] == self.name
      && volume["Labels"]["com.docker.compose.project"] == self.project
      && volume["Labels"]["com.docker.compose.volume"] == self.key
      && volume["Labels"]["io.gity.backup.restore"] == self.invocation
  }
}
impl<F: Fn(&[&str], &str) -> Result<Vec<u8>>> Drop for PendingVolume<'_, F> {
  fn drop(&mut self) {
    if !self.armed {
      return;
    }
    let proof = (self.run)(
      &["volume", "inspect", self.name],
      "inspect failed restore allocation",
    )
    .ok()
    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
    match proof {
      Some(proof) if self.owns(&proof) => {
        if (self.run)(
          &["volume", "rm", self.name],
          "remove failed restore allocation",
        )
        .is_err()
        {
          eprintln!("New restore volume cleanup failed: {}", self.name);
        }
      }
      Some(_) => {} // A raced or existing allocation belongs to somebody else.
      None => eprintln!(
        "Restore allocation ownership could not be checked; preserved volume: {}",
        self.name
      ),
    }
  }
}

pub fn verify_actual_mounts(config: &Value, containers: &[Value]) -> Result<()> {
  for container in containers {
    let service = container["Config"]["Labels"]["com.docker.compose.service"]
      .as_str()
      .context("service identity missing")?;
    let expected = config["services"][service]["volumes"]
      .as_array()
      .map(Vec::as_slice)
      .unwrap_or_default();
    for mount in container["Mounts"].as_array().into_iter().flatten() {
      let target = mount["Destination"]
        .as_str()
        .context("container mount target missing")?;
      let definition = expected
        .iter()
        .find(|entry| entry["target"] == target)
        .context("container has an undeclared persistent mount")?;
      match definition["type"].as_str() {
        Some("volume") => {
          let key = definition["source"]
            .as_str()
            .context("volume key missing")?;
          if mount["Type"] != "volume" || mount["Name"] != config["volumes"][key]["name"] {
            bail!("actual volume differs from supplied Compose configuration");
          }
        }
        Some("bind") => {
          let actual = std::path::Path::new(mount["Source"].as_str().context("bind path missing")?)
            .canonicalize()?;
          let declared = std::path::Path::new(
            definition["source"]
              .as_str()
              .context("bind source missing")?,
          )
          .canonicalize()?;
          if mount["Type"] != "bind" || actual != declared {
            bail!("actual bind differs from supplied Compose configuration");
          }
        }
        _ => bail!("unsupported actual mount type"),
      }
    }
  }
  Ok(())
}

#[cfg(test)]
#[path = "ownership_tests.rs"]
mod tests;
