use super::{BackupArgs, model};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::{
  path::Path,
  process::{Command, Stdio},
};
use uuid::Uuid;

pub fn command() -> Command {
  Command::new("docker")
}

pub fn output(command: &mut Command, action: &str) -> Result<Vec<u8>> {
  let result = command
    .output()
    .with_context(|| format!("execute {action}"))?;
  if !result.status.success() {
    // Docker errors may include interpolated environment or secrets; never echo them.
    bail!("{action} failed ({})", result.status);
  }
  Ok(result.stdout)
}

pub fn status(command: &mut Command, action: &str) -> Result<()> {
  command.stdout(Stdio::null()).stderr(Stdio::null());
  if !command
    .status()
    .with_context(|| format!("execute {action}"))?
    .success()
  {
    bail!("{action} failed; inspect the exact project's Docker state");
  }
  Ok(())
}

pub fn compose_config(root: &Path, args: &BackupArgs) -> Result<Value> {
  let mut cmd = command();
  cmd
    .current_dir(root)
    .args(["compose", "--project-name", &args.project, "--file"])
    .arg(&args.compose_file);
  if let Some(env) = &args.env_file {
    cmd.arg("--env-file").arg(env);
  }
  cmd.args(["--profile", "*", "config", "--format", "json"]);
  serde_json::from_slice(&output(&mut cmd, "resolve private Compose configuration")?)
    .context("decode private Compose configuration")
}

pub fn names(kind: &str, project: &str) -> Result<Vec<String>> {
  let bytes = output(
    command().args([
      kind,
      "ls",
      "--quiet",
      "--filter",
      &format!("label=com.docker.compose.project={project}"),
    ]),
    "list project resources",
  )?;
  Ok(
    String::from_utf8(bytes)?
      .lines()
      .map(str::to_owned)
      .collect(),
  )
}

pub fn project_containers(project: &str) -> Result<Vec<Value>> {
  let bytes = output(
    command().args([
      "ps",
      "--all",
      "--quiet",
      "--filter",
      &format!("label=com.docker.compose.project={project}"),
    ]),
    "list exact project containers",
  )?;
  let ids: Vec<_> = std::str::from_utf8(&bytes)?.split_whitespace().collect();
  if ids.is_empty() {
    return Ok(Vec::new());
  }
  serde_json::from_slice(&output(
    command().arg("inspect").args(ids),
    "inspect exact project containers",
  )?)
  .context("decode container metadata")
}

pub fn running_ids(containers: &[Value]) -> Vec<String> {
  containers
    .iter()
    .filter(|c| c["State"]["Running"].as_bool() == Some(true))
    .filter_map(|c| c["Id"].as_str().map(str::to_owned))
    .collect()
}

pub fn stop_project(project: &str, ids: &[String]) -> Result<()> {
  if !ids.is_empty() {
    status(
      command().args(["stop", "--time", "120"]).args(ids),
      "stop all project writers and stores",
    )?;
  }
  assert_stopped(project)
}

pub fn assert_stopped(project: &str) -> Result<()> {
  if !running_ids(&project_containers(project)?).is_empty() {
    bail!("project is not fully stopped; snapshot refused");
  }
  Ok(())
}

pub fn ensure_empty_project(project: &str) -> Result<()> {
  if !project_containers(project)?.is_empty()
    || !names("volume", project)?.is_empty()
    || !names("network", project)?.is_empty()
  {
    bail!("restore requires a new empty Compose project");
  }
  Ok(())
}

pub fn volume_exists(name: &str) -> Result<bool> {
  let bytes = output(
    command().args([
      "volume",
      "ls",
      "--format",
      "{{.Name}}",
      "--filter",
      &format!("name=^{name}$"),
    ]),
    "check target volume",
  )?;
  Ok(
    std::str::from_utf8(&bytes)?
      .lines()
      .any(|line| line == name),
  )
}

pub struct Helper {
  name: String,
  image: String,
}

impl Helper {
  pub fn new(image: &str) -> Self {
    Self {
      name: format!("gity-backup-{}", Uuid::now_v7().simple()),
      image: image.into(),
    }
  }

  pub fn command(&self) -> Command {
    let mut cmd = command();
    cmd.args([
      "run",
      "--rm",
      "--name",
      &self.name,
      "--label",
      "io.gity.backup.helper=true",
      "--network",
      "none",
      "--read-only",
      "--user",
      "0:0",
      "--cap-drop",
      "ALL",
      "--cap-add",
      "DAC_OVERRIDE",
      "--cap-add",
      "CHOWN",
      "--cap-add",
      "FOWNER",
    ]);
    cmd
  }

  pub fn image(&self) -> &str {
    &self.image
  }
}

impl Drop for Helper {
  fn drop(&mut self) {
    let _ = command()
      .args(["rm", "--force", &self.name])
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status();
  }
}

pub fn mount(kind: &str, source: &str, target: &str, readonly: bool) -> Result<String> {
  if source.contains([',', '\n', '\r']) || target.contains([',', '\n', '\r']) {
    bail!("Docker mount paths may not contain commas or line breaks");
  }
  Ok(format!(
    "type={kind},src={source},dst={target}{}",
    if readonly { ",readonly" } else { "" }
  ))
}

pub fn verify_helper(image: &str) -> Result<()> {
  let helper = Helper::new(image);
  let version = output(
    helper.command().args([helper.image(), "tar", "--version"]),
    "verify GNU tar helper",
  )?;
  if !version.starts_with(b"tar (GNU tar)") {
    bail!("backup helper must provide GNU tar with xattrs and ACL support");
  }
  Ok(())
}

pub fn archive(
  helper_image: &str,
  kind: &str,
  source: &str,
  policy: model::TarPolicy,
  path: &Path,
) -> Result<model::Archive> {
  let helper = Helper::new(helper_image);
  let mut cmd = helper.command();
  let directory = policy != model::TarPolicy::FileBind;
  let target = if directory {
    "/source"
  } else {
    "/source/payload"
  };
  // Reading trusted.overlay.* metadata requires SYS_ADMIN. Keep the source
  // read-only and retain the runtime default seccomp/AppArmor profiles.
  cmd
    .args(["--cap-add", "SYS_ADMIN"])
    .arg("--mount")
    .arg(mount(kind, source, target, true)?);
  cmd.args([
    helper.image(),
    "tar",
    "--format=pax",
    "--xattrs",
    "--xattrs-include=*",
    "--acls",
    "--numeric-owner",
  ]);
  policy.tar_exclusions(&mut cmd);
  cmd.args([
    "-cpf",
    "-",
    "-C",
    "/source",
    if directory { "." } else { "payload" },
  ]);
  let file = model::private_file(path)?;
  cmd
    .stdout(Stdio::from(file.try_clone()?))
    .stderr(Stdio::null());
  if !cmd
    .status()
    .context("stream offline volume archive")?
    .success()
  {
    bail!("archive helper failed; incomplete bundle must not be restored");
  }
  file.sync_all()?;
  model::identity(path)
}
