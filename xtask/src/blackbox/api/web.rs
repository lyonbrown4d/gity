use super::{Reporter, RunConfig};
use anyhow::{Context, Result, bail};
use reqwest::{StatusCode, blocking::Client, header::CONTENT_TYPE};
use serde::Deserialize;
use std::{collections::BTreeSet, path::Path};
use url::Url;

#[derive(Deserialize)]
struct VendorManifest {
  monaco_version: String,
  milkdown_version: String,
  files: Vec<VendorFile>,
}

#[derive(Deserialize)]
struct VendorFile {
  path: String,
}

pub(super) fn run(root: &Path, config: &RunConfig, reporter: &mut Reporter) -> Result<()> {
  let client = Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
  let base = Url::parse(&format!("{}/", config.http_url))?;
  let (index, bytes) = fetch(&client, &base, "/", "text/html")?;
  let html = String::from_utf8(bytes).context("published Web homepage is not UTF-8")?;
  if !html.to_ascii_lowercase().contains("<!doctype html")
    || !html.contains("Gity")
    || !html.contains("/vendor/editors.css")
  {
    bail!("published Web homepage is not the expected Gity HTML document");
  }

  let references = static_references(&html);
  let js = references
    .iter()
    .filter(|path| extension(path) == Some("js"))
    .cloned()
    .collect::<Vec<_>>();
  let wasm = references
    .iter()
    .filter(|path| extension(path) == Some("wasm"))
    .cloned()
    .collect::<Vec<_>>();
  if js.is_empty() || wasm.is_empty() {
    bail!("published homepage does not reference both its JavaScript loader and WASM module");
  }

  let mut checked = BTreeSet::new();
  checked.insert(index);
  for path in js.iter().chain(wasm.iter()) {
    let expected = if extension(path) == Some("wasm") {
      "application/wasm"
    } else {
      "application/javascript"
    };
    checked.insert(fetch(&client, &base, path, expected)?.0);
  }

  let manifest_path = root.join("apps/gity-web/vendor/manifest.toml");
  let manifest: VendorManifest = toml::from_str(
    &std::fs::read_to_string(&manifest_path)
      .with_context(|| format!("read vendor asset manifest {}", manifest_path.display()))?,
  )
  .context("parse vendor asset manifest")?;
  if manifest.monaco_version.trim().is_empty()
    || manifest.milkdown_version.trim().is_empty()
    || manifest.files.is_empty()
  {
    bail!("vendor asset manifest is missing editor versions or files");
  }
  for required in ["editors.css", "editors.js"] {
    if !manifest.files.iter().any(|file| file.path == required) {
      bail!("vendor asset manifest is missing {required}");
    }
  }
  for file in &manifest.files {
    let path = format!("/vendor/{}", file.path.trim_start_matches('/'));
    let expected = match extension(&file.path) {
      Some("css") => "text/css",
      Some("js" | "mjs") => "application/javascript",
      _ => bail!("unsupported vendor asset type in manifest: {}", file.path),
    };
    checked.insert(fetch(&client, &base, &path, expected)?.0);
  }

  reporter.pass(
    "published-web-static-assets",
    format!(
      "homepage HTML; {} JS loader(s), {} WASM module(s), and {} vendor assets (Monaco {}, Milkdown {}) returned non-empty resources with expected MIME types",
      js.len(),
      wasm.len(),
      manifest.files.len(),
      manifest.monaco_version,
      manifest.milkdown_version
    ),
  );
  Ok(())
}

fn fetch(client: &Client, base: &Url, path: &str, expected_mime: &str) -> Result<(Url, Vec<u8>)> {
  let url = base
    .join(path)
    .with_context(|| format!("resolve Web asset {path}"))?;
  if url.origin() != base.origin() {
    bail!("Web asset escaped the published site origin: {path}");
  }
  let response = client
    .get(url.clone())
    .send()
    .with_context(|| format!("request published Web asset {path}"))?;
  if response.status() != StatusCode::OK {
    bail!(
      "published Web asset {path} returned HTTP {}",
      response.status()
    );
  }
  let mime = response
    .headers()
    .get(CONTENT_TYPE)
    .and_then(|value| value.to_str().ok())
    .unwrap_or_default()
    .split(';')
    .next()
    .unwrap_or_default()
    .trim();
  let valid_mime = if expected_mime == "application/javascript" {
    matches!(
      mime.to_ascii_lowercase().as_str(),
      "application/javascript" | "text/javascript" | "application/ecmascript" | "text/ecmascript"
    )
  } else {
    mime.eq_ignore_ascii_case(expected_mime)
  };
  if !valid_mime {
    bail!("published Web asset {path} has MIME {mime}, expected {expected_mime}");
  }
  let bytes = response.bytes().context("read published Web asset")?;
  if bytes.is_empty() || (expected_mime != "text/html" && starts_with_html(&bytes)) {
    bail!("published Web asset {path} is empty or is an SPA fallback page");
  }
  Ok((url, bytes.to_vec()))
}

fn starts_with_html(bytes: &[u8]) -> bool {
  let prefix = String::from_utf8_lossy(&bytes[..bytes.len().min(128)]).to_ascii_lowercase();
  let prefix = prefix.trim_start();
  prefix.starts_with("<!doctype html") || prefix.starts_with("<html")
}

fn static_references(html: &str) -> BTreeSet<String> {
  let mut paths = BTreeSet::new();
  let mut quote = None;
  let mut start = 0;
  for (index, character) in html.char_indices() {
    match (quote, character) {
      (None, '"' | '\'') => {
        quote = Some(character);
        start = index + character.len_utf8();
      }
      (Some(expected), current) if expected == current => {
        let value = html[start..index].trim();
        let value = value.split(['?', '#']).next().unwrap_or_default();
        if matches!(extension(value), Some("js" | "wasm"))
          && !value.starts_with("data:")
          && !value.contains("://")
        {
          paths.insert(value.to_owned());
        }
        quote = None;
      }
      _ => {}
    }
  }
  paths
}

fn extension(path: &str) -> Option<&str> {
  path.rsplit_once('.')?.1.split(['?', '#', '/']).next()
}
