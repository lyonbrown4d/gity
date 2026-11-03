use super::{Reporter, RunConfig};
use anyhow::{Context, Result, bail};
use reqwest::{StatusCode, blocking::Client, header::CONTENT_TYPE};
use std::{collections::BTreeSet, path::Path};
use url::Url;

pub(super) fn run(_root: &Path, config: &RunConfig, reporter: &mut Reporter) -> Result<()> {
  let client = Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
  let base = Url::parse(&format!("{}/", config.http_url))?;
  let (index, bytes) = fetch(&client, &base, "/", "text/html")?;
  let html = String::from_utf8(bytes).context("published Web homepage is not UTF-8")?;
  if !html.to_ascii_lowercase().contains("<!doctype html")
    || !html.contains("Gity")
    || !html.contains("id=\"root\"")
  {
    bail!("published Web homepage is not the expected Gity HTML document");
  }

  let references = static_references(&html);
  let js = references
    .iter()
    .filter(|path| extension(path) == Some("js"))
    .cloned()
    .collect::<Vec<_>>();
  let css = references
    .iter()
    .filter(|path| extension(path) == Some("css"))
    .cloned()
    .collect::<Vec<_>>();
  if js.is_empty() || css.is_empty() {
    bail!("published React homepage must reference its built JavaScript and CSS assets");
  }

  let mut checked = BTreeSet::new();
  checked.insert(index);
  let mut pending = references
    .into_iter()
    .map(|path| (base.clone(), path))
    .collect::<Vec<_>>();
  let mut editors = false;
  while let Some((parent, path)) = pending.pop() {
    let url = parent.join(&path)?;
    if checked.contains(&url) {
      continue;
    }
    let expected = match extension(&path) {
      Some("css") => "text/css",
      Some("js" | "mjs") => "application/javascript",
      _ => bail!("unsupported Vite asset: {path}"),
    };
    if url.origin() != base.origin() {
      bail!("Web asset escaped the published site origin: {path}");
    }
    let (url, bytes) = fetch(&client, &parent, &path, expected)?;
    editors |= url.path().contains("/editor-");
    if expected == "application/javascript" {
      let code = std::str::from_utf8(&bytes).context("Web JavaScript is not UTF-8")?;
      for reference in static_references(code) {
        // Vite preload maps use root-relative assets/ paths; static imports
        // use sibling chunk names. Both must resolve to published local files.
        let parent = if reference.starts_with("assets/") {
          base.clone()
        } else {
          url.clone()
        };
        pending.push((parent, reference));
      }
    }
    checked.insert(url);
  }
  if !editors {
    bail!("published Vite bundle does not include the local editor chunk");
  }

  reporter.pass(
    "published-web-static-assets",
    format!(
      "React homepage and {} local Vite JS/CSS assets, including editor chunks, returned non-empty resources with expected MIME types",
      checked.len() - 1
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
        if matches!(extension(value), Some("js" | "mjs" | "css"))
          && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_/.:".contains(&byte))
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
  path
    .split(['?', '#'])
    .next()?
    .rsplit_once('.')
    .map(|(_, ext)| ext)
}

#[cfg(test)]
#[path = "web_tests.rs"]
mod tests;
