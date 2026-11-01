use super::super::Clients;
use super::process;
use anyhow::{Context, Result, bail};
use reqwest::Url;
use std::{
  path::Path,
  process::Command,
  time::{Duration, Instant},
};

pub(super) struct TlsProxy {
  name: String,
  certificate: String,
  owns_container: bool,
}

impl TlsProxy {
  pub(super) fn start(context: &Clients, work: &Path) -> Result<Self> {
    let name = format!("gity-bb-go-proxy-{}", context.suffix);
    let tls = work.join("tls-proxy");
    std::fs::create_dir_all(&tls)?;
    let certificate = tls.join("server.crt");
    let key = tls.join("server.key");
    create_certificate(&name, &certificate, &key)?;
    write_nginx_config(context, &name, &tls)?;

    let mut proxy = Self {
      name,
      certificate: "/work/tls-proxy/server.crt".to_owned(),
      owns_container: false,
    };
    let mut start = Command::new("docker");
    start.args(["run", "--detach", "--name", &proxy.name, "--network"]);
    start.arg(&context.network);
    start
      .arg("--volume")
      .arg(format!("{}:/etc/nginx/tls:ro", tls.display()))
      .arg("--volume")
      .arg(format!(
        "{}:/etc/nginx/conf.d/default.conf:ro",
        tls.join("default.conf").display()
      ))
      .arg("nginx:alpine");
    process::run_output(&mut start, "start task-owned Go HTTPS proxy")?;
    proxy.owns_container = true;
    proxy.wait_ready()?;
    Ok(proxy)
  }

  pub(super) fn name(&self) -> &str {
    &self.name
  }

  pub(super) fn certificate(&self) -> &str {
    &self.certificate
  }

  fn wait_ready(&self) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
      let running = process::run_output(
        Command::new("docker").args(["inspect", "--format", "{{.State.Running}}", &self.name]),
        "check Go HTTPS proxy container state",
      )?;
      if String::from_utf8(running.stdout)?.trim() != "true" {
        bail!("task-owned Go HTTPS proxy exited before becoming ready");
      }
      if process::run_output(
        Command::new("docker").args(["exec", &self.name, "nginx", "-t"]),
        "validate Go HTTPS proxy configuration",
      )
      .is_ok()
      {
        return Ok(());
      }
      if Instant::now() >= deadline {
        bail!("task-owned Go HTTPS proxy did not become ready within 20 seconds");
      }
      std::thread::sleep(Duration::from_millis(250));
    }
  }
}

impl Drop for TlsProxy {
  fn drop(&mut self) {
    if self.owns_container {
      let _ = Command::new("docker")
        .args(["rm", "--force", &self.name])
        .output();
    }
  }
}

fn create_certificate(name: &str, certificate: &Path, key: &Path) -> Result<()> {
  let subject_alt_name = format!("subjectAltName=DNS:{name}");
  process::run_output(
    Command::new("openssl").args([
      "req",
      "-x509",
      "-nodes",
      "-newkey",
      "rsa:2048",
      "-keyout",
      key
        .to_str()
        .context("Go HTTPS proxy key path is not UTF-8")?,
      "-out",
      certificate
        .to_str()
        .context("Go HTTPS proxy certificate path is not UTF-8")?,
      "-days",
      "1",
      "-subj",
      "/CN=Gity blackbox Go proxy",
      "-addext",
      &subject_alt_name,
    ]),
    "generate temporary Go HTTPS proxy certificate",
  )?;
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(key, std::fs::Permissions::from_mode(0o600))?;
  }
  Ok(())
}

fn write_nginx_config(context: &Clients, name: &str, tls_dir: &Path) -> Result<()> {
  let api = Url::parse(&context.api_url).context("parse Go proxy upstream URL")?;
  let host = api
    .host_str()
    .ok_or_else(|| anyhow::anyhow!("Go proxy upstream has no host"))?;
  let upstream = match api.port() {
    Some(port) => format!("{host}:{port}"),
    None => host.to_owned(),
  };
  let config = format!(
    "server {{\n  listen 443 ssl;\n  server_name {name};\n  ssl_certificate /etc/nginx/tls/server.crt;\n  ssl_certificate_key /etc/nginx/tls/server.key;\n  location / {{\n    proxy_pass http://{upstream};\n    proxy_set_header Host $proxy_host;\n    proxy_set_header Authorization $http_authorization;\n    proxy_set_header Connection \"\";\n  }}\n}}\n"
  );
  std::fs::write(tls_dir.join("default.conf"), config)
    .context("write temporary Go HTTPS proxy configuration")
}
