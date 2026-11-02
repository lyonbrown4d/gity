use super::{
  RestoreArgs, RestoreResult, docker,
  model::{self, Manifest},
};
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::{
  collections::BTreeMap,
  path::{Path, PathBuf},
  process::Stdio,
};

struct Allocation {
  volumes: Vec<String>,
  assets: PathBuf,
  helper: String,
  committed: bool,
}

impl Drop for Allocation {
  fn drop(&mut self) {
    if self.committed {
      return;
    }
    for volume in &self.volumes {
      let _ = docker::status(
        docker::command().args(["volume", "rm", volume]),
        "remove newly allocated restore volume",
      );
    }
    if self.assets.exists() {
      // Files retain original UID/mode. A root helper cleans only the directory
      // this invocation created, never a pre-existing host directory.
      let helper = docker::Helper::new(&self.helper);
      let mut cmd = helper.command();
      if let Ok(mount) = docker::mount("bind", &self.assets.to_string_lossy(), "/cleanup", false) {
        let _ = docker::status(
          cmd.arg("--mount").arg(mount).args([
            helper.image(),
            "find",
            "/cleanup",
            "-mindepth",
            "1",
            "-delete",
          ]),
          "clean new restore assets",
        );
      }
      let _ = std::fs::remove_dir_all(&self.assets);
    }
  }
}

pub fn run(
  args: &RestoreArgs,
  bundle: &Path,
  manifest: &Manifest,
  mut config: Value,
) -> Result<RestoreResult> {
  model::create_private_dir(&args.secrets_dir)?;
  let assets = args.secrets_dir.canonicalize()?;
  let mut allocated = Allocation {
    volumes: Vec::new(),
    assets: assets.clone(),
    helper: args.helper_image.clone(),
    committed: false,
  };
  let mut bind_paths = BTreeMap::new();
  for bind in &manifest.binds {
    let target = assets.join(&bind.id);
    model::create_private_dir(&target)?;
    extract(
      &args.helper_image,
      "bind",
      target.to_str().context("asset path must be UTF-8")?,
      &bundle.join(&bind.archive.file),
      if bind.directory {
        model::TarPolicy::Directory
      } else {
        model::TarPolicy::FileBind
      },
    )?;
    bind_paths.insert(
      bind.source.clone(),
      if bind.directory {
        target
      } else {
        target.join("payload")
      },
    );
  }
  let invocation = uuid::Uuid::now_v7().to_string();
  for volume in &manifest.volumes {
    let name = super::ownership::create_volume(&args.project, &volume.key, &invocation)?;
    allocated.volumes.push(name.clone());
    if let Some(archive) = &volume.archive {
      extract(
        &args.helper_image,
        "volume",
        &name,
        &bundle.join(&archive.file),
        model::TarPolicy::volume(&config, &volume.key),
      )?;
    }
    config["volumes"][&volume.key] = json!({"name":name, "external":true});
  }
  remap(&mut config, &args.project, &bind_paths)?;
  let compose_file = assets.join("compose.json");
  escape_interpolation(&mut config);
  model::write_private(&compose_file, &serde_json::to_vec_pretty(&config)?)?;
  allocated.committed = true;
  println!(
    "Offline restore prepared; services remain stopped. Private Compose file: {}",
    compose_file.display()
  );
  Ok(RestoreResult {
    compose_file,
    project: args.project.clone(),
  })
}

pub(super) fn extract(
  image: &str,
  kind: &str,
  source: &str,
  archive: &Path,
  policy: model::TarPolicy,
) -> Result<()> {
  let helper = docker::Helper::new(image);
  let mut cmd = helper.command();
  // Stream from the client, just as capture streams to it. The archive need
  // not exist in the Docker daemon's host filesystem (remote/DIND daemons).
  cmd
    .args([
      "--cap-add",
      "SYS_ADMIN",
      "--cap-add",
      "SETFCAP",
      "--cap-add",
      "FSETID",
    ])
    .arg("--interactive")
    // Only extraction recreates verified overlay2 character-device 0:0 whiteouts.
    // Capture and cleanup helpers retain the narrower shared capability set.
    .args(["--cap-add", "MKNOD"])
    .arg("--mount")
    .arg(docker::mount(kind, source, "/restore", false)?)
    .args([
      helper.image(),
      "tar",
      "--xattrs",
      "--xattrs-include=*",
      "--acls",
      "--numeric-owner",
      "--same-owner",
      "--same-permissions",
    ]);
  policy.tar_exclusions(&mut cmd);
  cmd
    .args(["-xpf", "-", "-C", "/restore"])
    .stdin(Stdio::from(std::fs::File::open(archive)?));
  docker::status(&mut cmd, "extract into isolated new restore resource")
}

pub fn remap(config: &mut Value, project: &str, binds: &BTreeMap<String, PathBuf>) -> Result<()> {
  config["name"] = Value::String(project.into());
  let services = config["services"]
    .as_object_mut()
    .context("Compose services missing")?;
  for service in services.values_mut() {
    let object = service.as_object_mut().context("invalid Compose service")?;
    object.remove("build");
    object.remove("container_name");
    object.insert("pull_policy".into(), Value::String("never".into()));
    for mount in object
      .get_mut("volumes")
      .and_then(Value::as_array_mut)
      .into_iter()
      .flatten()
    {
      if mount["type"] == "bind" {
        let old = mount["source"]
          .as_str()
          .context("bind source missing")?
          .to_owned();
        let new = binds
          .get(&old)
          .context("missing restored bind mapping")?
          .to_str()
          .context("restored bind path must be UTF-8")?;
        if mount["target"] == old {
          mount["target"] = Value::String(new.into());
        }
        mount["source"] = Value::String(new.into());
        if let Some(bind) = mount.get_mut("bind").and_then(Value::as_object_mut) {
          bind.insert("create_host_path".into(), Value::Bool(false));
        }
      }
    }
    // Runner work-root occurs in environment and command as well as mount target.
    replace_paths(service, binds);
  }
  if let Some(networks) = config["networks"].as_object_mut() {
    for (key, network) in networks {
      if network["external"] == true {
        bail!("external networks are unsupported for isolated restore");
      }
      network["name"] = Value::String(format!("{project}_{key}"));
    }
  }
  Ok(())
}

fn replace_paths(value: &mut Value, binds: &BTreeMap<String, PathBuf>) {
  match value {
    Value::String(s) => {
      if let Some(path) = binds.get(s) {
        *s = path.to_string_lossy().into_owned();
      }
    }
    Value::Array(items) => {
      for item in items {
        replace_paths(item, binds);
      }
    }
    Value::Object(items) => {
      for item in items.values_mut() {
        replace_paths(item, binds);
      }
    }
    _ => {}
  }
}

fn escape_interpolation(value: &mut Value) {
  match value {
    Value::String(s) => *s = s.replace('$', "$$"),
    Value::Array(items) => {
      for item in items {
        escape_interpolation(item);
      }
    }
    Value::Object(items) => {
      for item in items.values_mut() {
        escape_interpolation(item);
      }
    }
    _ => {}
  }
}
