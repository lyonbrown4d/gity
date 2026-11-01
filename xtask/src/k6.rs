use anyhow::{Context, Result, bail};
use clap::Args;
use std::{
  path::{Path, PathBuf},
  process::Command,
  thread,
  time::Duration,
};

#[derive(Clone, Args)]
pub struct K6Args {
  #[arg(long)]
  build: bool,
  #[arg(long)]
  docker_k6: bool,
  #[arg(long)]
  no_compose: bool,
  #[arg(long, default_value = "")]
  base_url: String,
  #[arg(long, default_value = "")]
  docker_base_url: String,
  #[arg(long, default_value_t = 18080)]
  port: u16,
  #[arg(long, default_value = "gity-blackbox")]
  project: String,
  #[arg(long, default_value_t = 8)]
  vus: u32,
  #[arg(long, default_value = "30s")]
  duration: String,
  #[arg(long, default_value_t = 1000)]
  p95: u32,
  #[arg(long, default_value_t = 12)]
  write_percent: u32,
  #[arg(long, default_value = "perf/k6/common-api.js")]
  script: PathBuf,
}

pub fn run(root: &Path, args: K6Args) -> Result<()> {
  if args.vus == 0 || args.duration.is_empty() || args.p95 == 0 || args.write_percent > 100 {
    bail!("vus and p95 must be positive, duration non-empty, and write-percent at most 100");
  }
  let script = if args.script.is_absolute() {
    args.script
  } else {
    root.join(args.script)
  };
  if !script.is_file() {
    bail!("k6 script does not exist: {}", script.display());
  }
  let base = if args.base_url.is_empty() {
    format!("http://127.0.0.1:{}", args.port)
  } else {
    args.base_url
  };
  if !args.no_compose {
    compose_up(root, &args.project, args.port, args.build)?;
    wait_for_server(&base, &args.project, root)?;
  }
  let local = !args.docker_k6 && command_exists("k6");
  if local {
    println!("Running k6 against {base}");
    let status = Command::new("k6")
      .current_dir(root)
      .arg("run")
      .arg(&script)
      .env("GITY_K6_BASE_URL", &base)
      .env("GITY_K6_VUS", args.vus.to_string())
      .env("GITY_K6_DURATION", &args.duration)
      .env("GITY_K6_P95_MS", args.p95.to_string())
      .env("GITY_K6_WRITE_PERCENT", args.write_percent.to_string())
      .status()
      .context("start k6")?;
    if !status.success() {
      bail!("k6 failed with {status}");
    }
    return Ok(());
  }
  let use_host_network = cfg!(target_os = "linux") && args.docker_base_url.is_empty();
  let docker_base = if !args.docker_base_url.is_empty() {
    args.docker_base_url
  } else if use_host_network {
    base.clone()
  } else {
    docker_reachable(&base, args.port)
  };
  println!("Running Docker k6 against {docker_base}");
  let parent = script.parent().context("k6 script parent directory")?;
  let filename = script.file_name().context("k6 script filename")?;
  let mount = format!("{}:/scripts:ro", parent.display());
  let container_script = format!("/scripts/{}", filename.to_string_lossy());
  let mut docker = Command::new("docker");
  docker.current_dir(root).args(["run", "--rm", "-i"]);
  if use_host_network {
    docker.arg("--network=host");
  }
  let status = docker
    .args(["-e", &format!("GITY_K6_BASE_URL={docker_base}")])
    .args(["-e", &format!("GITY_K6_VUS={}", args.vus)])
    .args(["-e", &format!("GITY_K6_DURATION={}", args.duration)])
    .args(["-e", &format!("GITY_K6_P95_MS={}", args.p95)])
    .args([
      "-e",
      &format!("GITY_K6_WRITE_PERCENT={}", args.write_percent),
    ])
    .args(["-v", &mount, "grafana/k6", "run", &container_script])
    .status()
    .context("start Docker k6")?;
  if !status.success() {
    bail!("Docker k6 failed with {status}");
  }
  Ok(())
}

fn compose_up(root: &Path, project: &str, port: u16, build: bool) -> Result<()> {
  let mut command = Command::new("docker");
  command
    .current_dir(root)
    .args([
      "compose",
      "-p",
      project,
      "-f",
      "docker-compose.blackbox.yaml",
      "up",
      "-d",
    ])
    .env("GITY_BLACKBOX_WEB_PORT", port.to_string());
  if build {
    command.arg("--build");
  }
  let status = command
    .status()
    .context("start blackbox Docker Compose stack")?;
  if !status.success() {
    bail!("docker compose up failed with {status}");
  }
  Ok(())
}

fn wait_for_server(base: &str, project: &str, root: &Path) -> Result<()> {
  let client = reqwest::blocking::Client::builder()
    .timeout(Duration::from_secs(3))
    .build()?;
  let mut last = String::new();
  for _ in 0..60 {
    let health = client
      .get(format!("{}/api/health", base.trim_end_matches('/')))
      .send()
      .and_then(|r| r.error_for_status());
    let home = client
      .get(format!("{}/", base.trim_end_matches('/')))
      .send()
      .and_then(|r| r.error_for_status());
    if health.is_ok() && home.is_ok() {
      return Ok(());
    }
    last = health
      .err()
      .or_else(|| home.err())
      .map(|e| e.to_string())
      .unwrap_or_default();
    thread::sleep(Duration::from_secs(2));
  }
  let _ = Command::new("docker")
    .current_dir(root)
    .args([
      "compose",
      "-p",
      project,
      "-f",
      "docker-compose.blackbox.yaml",
      "logs",
    ])
    .status();
  bail!("blackbox server did not become healthy: {last}")
}

fn command_exists(program: &str) -> bool {
  std::env::var_os("PATH").is_some_and(|paths| {
    std::env::split_paths(&paths).any(|dir| {
      let candidate = dir.join(program);
      candidate.is_file()
    })
  })
}

fn docker_reachable(raw: &str, port: u16) -> String {
  match url::Url::parse(raw) {
    Ok(mut url) if matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1")) => {
      let _ = url.set_host(Some("host.docker.internal"));
      if url.port().is_none() {
        let _ = url.set_port(Some(port));
      }
      url.to_string().trim_end_matches('/').to_owned()
    }
    _ => format!("http://host.docker.internal:{port}"),
  }
}
