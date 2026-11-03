use super::super::{Base, BlackboxArgs, Mode, Reporter, Suite, process};
use super::configure_compose;
use anyhow::{Context, Result, bail};
use std::{path::Path, process::Command};

pub fn capture_images(_root: &Path, args: &BlackboxArgs, reporter: &mut Reporter) -> Result<()> {
  let containers = project_containers(&args.project)?;
  let mut images = std::collections::BTreeSet::new();
  for container in &containers {
    let output = process::run_output(
      Command::new("docker").args(["inspect", "--format", "{{.Image}}", container]),
      "inspect black-box container image ID",
    )?;
    images.insert(String::from_utf8(output.stdout)?.trim().to_owned());
  }
  if images.is_empty() {
    bail!("black-box Compose stack has no inspectable images");
  }
  let mut details = Vec::new();
  for id in images {
    let inspected = process::run_output(
      Command::new("docker").args([
        "image",
        "inspect",
        "--format",
        "{{.Id}} {{.Os}} {{.Architecture}}",
        id.as_str(),
      ]),
      "inspect black-box image platform",
    )?;
    details.push(String::from_utf8(inspected.stdout)?.trim().to_owned());
  }
  let roles = capture_role_os(args)?;
  let stats = capture_idle_stats(&args.project)?;
  reporter.pass(
    "compose-image-platforms",
    format!(
      "base={} images={} roles={}",
      base_name(args.base),
      details.join(","),
      roles.join(",")
    ),
  );
  reporter.pass("idle-container-stats", stats);
  Ok(())
}

pub fn capture_failure_logs(
  root: &Path,
  args: &BlackboxArgs,
  evidence_path: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let mut command = Command::new("docker");
  configure_compose(&mut command, root, args)?;
  command.args(["logs", "--no-color", "--timestamps"]);
  let output = command
    .output()
    .context("capture black-box Compose failure logs")?;
  let mut logs = String::from_utf8_lossy(&output.stdout).into_owned();
  logs.push_str(&String::from_utf8_lossy(&output.stderr));
  let path = evidence_path.with_extension("logs.txt");
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).context("create failure-log directory")?;
  }
  std::fs::write(&path, reporter.redact(&logs)).context("write redacted Compose logs")?;
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
  }
  reporter.failure_logs(path);
  Ok(())
}

fn capture_role_os(args: &BlackboxArgs) -> Result<Vec<String>> {
  let services: &[&str] = match args.mode {
    Mode::Standalone => &["gity"],
    Mode::Distributed => &[
      "api",
      "git-server",
      "git-node",
      "git-node-2",
      "worker",
      "gateway",
      "registry",
    ],
  };
  let mut services = services.to_vec();
  if args.mode == Mode::Distributed && args.suite == Suite::Full {
    services.push("runner");
  }
  let expected = base_name(args.base);
  let mut details = Vec::new();
  for service in &services {
    let project_label = format!("label=com.docker.compose.project={}", args.project);
    let service_label = format!("label=com.docker.compose.service={service}");
    let mut query = Command::new("docker");
    query.args([
      "ps",
      "--all",
      "--quiet",
      "--filter",
      &project_label,
      "--filter",
      &service_label,
    ]);
    let output = process::run_output(&mut query, "locate black-box Gity role container")?;
    let id = String::from_utf8(output.stdout)?
      .lines()
      .map(str::trim)
      .find(|line| !line.is_empty())
      .ok_or_else(|| anyhow::anyhow!("black-box Gity role {service} has no container"))?
      .to_owned();
    let os = process::run_output(
      Command::new("docker").args(["exec", &id, "cat", "/etc/os-release"]),
      "read Gity role /etc/os-release",
    )?;
    let release = String::from_utf8(os.stdout).context("OS release is not UTF-8")?;
    let actual = release
      .lines()
      .find_map(|line| line.strip_prefix("ID="))
      .map(|id| id.trim_matches('"'))
      .ok_or_else(|| anyhow::anyhow!("Gity role {service} omitted OS ID"))?;
    if actual != expected {
      bail!("Gity role {service} runs {actual}, expected {expected}");
    }
    let uid = process::run_output(
      Command::new("docker").args(["exec", &id, "id", "-u"]),
      "read Gity role runtime UID",
    )?;
    if String::from_utf8(uid.stdout)?.trim() != "10001" {
      bail!("Gity role {service} must run as UID 10001");
    }
    details.push(format!("{service}={actual},uid=10001"));
  }
  Ok(details)
}

fn capture_idle_stats(project: &str) -> Result<String> {
  let all = project_containers(project)?;
  let running = all
    .into_iter()
    .filter(|id| {
      Command::new("docker")
        .args(["inspect", "--format", "{{.State.Running}}", id])
        .output()
        .is_ok_and(|output| String::from_utf8_lossy(&output.stdout).trim() == "true")
    })
    .collect::<Vec<_>>();
  let mut command = Command::new("docker");
  command.args([
    "stats",
    "--no-stream",
    "--format",
    "{{.Name}} {{.CPUPerc}} {{.MemUsage}} {{.MemPerc}}",
  ]);
  if running.is_empty() {
    bail!("black-box Compose stack has no running containers for idle stats");
  }
  command.args(running);
  let stats = process::run_output(&mut command, "capture idle black-box container stats")?;
  String::from_utf8(stats.stdout)
    .map(|output| output.trim().to_owned())
    .context("Docker container stats are not UTF-8")
}

fn project_containers(project: &str) -> Result<Vec<String>> {
  let label = format!("label=com.docker.compose.project={project}");
  let output = process::run_output(
    Command::new("docker").args(["ps", "--all", "--quiet", "--filter", &label]),
    "list black-box Compose containers",
  )?;
  let ids = String::from_utf8(output.stdout).context("Docker container IDs are not UTF-8")?;
  let ids = ids
    .lines()
    .map(str::trim)
    .filter(|id| !id.is_empty())
    .map(str::to_owned)
    .collect::<Vec<_>>();
  if ids.is_empty() {
    bail!("black-box Compose stack has no inspectable containers");
  }
  Ok(ids)
}

fn base_name(base: Base) -> &'static str {
  match base {
    Base::Debian => "debian",
    Base::Alpine => "alpine",
  }
}
