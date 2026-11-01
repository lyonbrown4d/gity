use super::{BlackboxArgs, Reporter, api, process};
use anyhow::{Context, Result};
use std::{
  path::{Path, PathBuf},
  process::Command,
};

mod cargo;
mod go;
mod maven;
mod npm;
mod nuget;
mod pypi;

pub struct Clients {
  pub network: String,
  download_proxy: Option<url::Url>,
  pub api_url: String,
  pub host_api_url: String,
  pub project_id: String,
  pub token: String,
  pub suffix: String,
  pub work: PathBuf,
}

pub fn run(
  args: &BlackboxArgs,
  scenario: &api::Scenario,
  work: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let mut command = Command::new("docker");
  command.args([
    "network",
    "ls",
    "--filter",
    &format!("label=com.docker.compose.project={}", args.project),
    "--filter",
    "label=com.docker.compose.network=default",
    "--format",
    "{{.Name}}",
  ]);
  let output = process::run_output(&mut command, "locate isolated black-box Compose network")?;
  let networks = String::from_utf8(output.stdout).context("Docker network names are not UTF-8")?;
  let network = networks
    .lines()
    .map(str::trim)
    .find(|line| !line.is_empty())
    .ok_or_else(|| anyhow::anyhow!("black-box Compose network was not created"))?
    .to_owned();
  let host = if args.mode == super::Mode::Standalone {
    "gity"
  } else {
    "gateway"
  };
  let context = Clients {
    network,
    download_proxy: super::proxy::read()?,
    api_url: format!("http://{host}:8080/api/v1"),
    host_api_url: args.api_url.trim_end_matches('/').to_owned(),
    project_id: scenario.project_id.clone(),
    token: scenario.owner_token.clone(),
    suffix: scenario.suffix.clone(),
    work: work.join("native-clients"),
  };
  std::fs::create_dir_all(&context.work)?;

  go::run(&context, reporter)?;
  cargo::run(&context, reporter)?;
  npm::run(&context, reporter)?;
  pypi::run(&context, reporter)?;
  maven::run(&context, reporter)?;
  nuget::run(&context, reporter)?;
  Ok(())
}

pub(super) fn container(
  context: &Clients,
  image: &str,
  work: &Path,
  environment: &[(&str, &str)],
) -> Command {
  let mut command = Command::new("docker");
  command
    .args(["run", "--rm", "--network", &context.network, "--volume"])
    .arg(format!("{}:/work", work.display()))
    .args(["--entrypoint", "sh", "--workdir", "/work"]);
  if let Some(proxy) = &context.download_proxy {
    let bypass = format!(
      "gity,gateway,localhost,127.0.0.1,gity-bb-go-proxy-{}",
      context.suffix
    );
    for key in ["HTTP_PROXY", "HTTPS_PROXY", "http_proxy", "https_proxy"] {
      command.env(key, proxy.as_str()).arg("--env").arg(key);
    }
    command.env("NO_PROXY", &bypass).args(["--env", "NO_PROXY"]);
    command.env("no_proxy", &bypass).args(["--env", "no_proxy"]);
    let java = format!(
      "-Dhttp.proxyHost={host} -Dhttp.proxyPort={port} -Dhttps.proxyHost={host} -Dhttps.proxyPort={port} -Dhttp.nonProxyHosts=gity|gateway|localhost|127.*",
      host = proxy.host_str().unwrap(),
      port = proxy.port_or_known_default().unwrap()
    );
    command
      .arg("--env")
      .arg(format!("JAVA_TOOL_OPTIONS={java}"));
  }
  for (key, value) in environment {
    command.env(key, value).arg("--env").arg(key);
  }
  command.arg(image);
  command
}
