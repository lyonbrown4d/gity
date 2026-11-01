use super::super::{Reporter, process};
use super::{Clients, container};
use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::{StatusCode, Url, blocking::Client};
use serde_json::json;
use std::{fs, path::Path};

#[path = "tls_proxy.rs"]
mod tls_proxy;

pub(super) fn run(context: &Clients, reporter: &mut Reporter) -> Result<()> {
  let root = context.work.join("go");
  let module = format!("example.com/gity/bb-{}", context.suffix);
  let version = "v0.1.0";
  fs::create_dir_all(root.join("consumer"))?;
  let (info_bytes, mod_bytes, zip_bytes) = make_files(context, &root, &module, version)?;
  let client = Client::builder()
    .timeout(std::time::Duration::from_secs(60))
    .build()?;
  upload(
    context,
    &client,
    &module,
    version,
    "v0.1.0.info",
    &info_bytes,
    "application/json",
  )?;
  upload(
    context,
    &client,
    &module,
    version,
    "v0.1.0.mod",
    &mod_bytes,
    "text/plain",
  )?;
  upload(
    context,
    &client,
    &module,
    version,
    "v0.1.0.zip",
    &zip_bytes,
    "application/zip",
  )?;
  let proxy = tls_proxy::TlsProxy::start(context, &root)?;
  let mut proxy_url = Url::parse(&format!(
    "https://{}/api/v1/projects/{}/packages/go",
    proxy.name(),
    context.project_id
  ))?;
  proxy_url
    .set_username("token")
    .map_err(|_| anyhow::anyhow!("set Go proxy username"))?;
  proxy_url
    .set_password(Some(&context.token))
    .map_err(|_| anyhow::anyhow!("set Go proxy token"))?;
  let mut command = container(
    context,
    "golang:1.25.0-bookworm",
    &root,
    &[
      ("GOPROXY", proxy_url.as_str()),
      ("GONOSUMDB", &module),
      ("SSL_CERT_FILE", proxy.certificate()),
    ],
  );
  command.args([
    "-ec",
    "cd /work/consumer && go mod tidy && go mod download && output=$(go run .) && test \"$output\" = 'go proxy roundtrip'",
  ]);
  process::run_status(&mut command, "native Go module proxy download and build")?;
  reporter.pass(
    "native-go-proxy-download",
    format!("downloaded and built {module}@{version} with Go"),
  );
  Ok(())
}

fn make_files(
  _context: &Clients,
  root: &Path,
  module: &str,
  version: &str,
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
  use std::io::Write;
  let file = fs::File::create(root.join("module.zip"))?;
  let mut zip = zip::ZipWriter::new(file);
  let options = zip::write::SimpleFileOptions::default();
  zip.start_file(format!("{module}@{version}/go.mod"), options)?;
  write!(zip, "module {module}\n\ngo 1.25\n")?;
  zip.start_file(format!("{module}@{version}/marker.go"), options)?;
  writeln!(
    zip,
    "package gitybb\nfunc Marker() string {{ return \"go proxy roundtrip\" }}"
  )?;
  zip.finish()?;
  let mod_content = format!("module {module}\n\ngo 1.25\n");
  let info_content = format!("{{\"Version\":\"{version}\",\"Time\":\"2026-11-01T00:00:00Z\"}}\n");
  fs::write(
    root.join("consumer/go.mod"),
    format!("module blackbox-consumer\n\ngo 1.25\n\nrequire {module} {version}\n"),
  )?;
  fs::write(
    root.join("consumer/main.go"),
    format!(
      "package main\nimport(\"fmt\"; \"{module}\")\nfunc main() {{ fmt.Print(gitybb.Marker()) }}\n"
    ),
  )?;
  Ok((
    info_content.into_bytes(),
    mod_content.into_bytes(),
    fs::read(root.join("module.zip"))?,
  ))
}

fn upload(
  context: &Clients,
  client: &Client,
  module: &str,
  version: &str,
  file: &str,
  bytes: &[u8],
  content_type: &str,
) -> Result<()> {
  let path = format!(
    "{}/projects/{}/packages/files",
    context.host_api_url, context.project_id
  );
  let response = client
    .post(path)
    .bearer_auth(&context.token)
    .json(&json!({
      "type":"go", "name":module, "version":version, "file_name":file,
      "content_type":content_type, "content_base64":STANDARD.encode(bytes)
    }))
    .send()
    .context("upload Go proxy fixture through package API")?;
  if response.status() != StatusCode::OK {
    bail!(
      "Go proxy artifact upload expected HTTP 200, got {}",
      response.status()
    );
  }
  Ok(())
}
