//! Front-end bundle embedded at compile time so a deployment ships one binary.
//!
//! `dist/web` is a Vite build: entry files live at the root, hashed chunks under
//! `assets/`, and brotli sidecars (`<file>.br`) are produced by `xtask precompress`.
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../../dist/web"]
struct WebAssets;

/// One embedded asset together with the headers it must be served with.
pub struct Asset {
  pub body: Vec<u8>,
  pub content_type: &'static str,
  pub encoding: Option<&'static str>,
}

/// Returns the embedded body of `path` and its content type.
pub fn get(path: &str) -> Option<(Vec<u8>, &'static str)> {
  let file = WebAssets::get(path)?;
  Some((file.data.into_owned(), content_type(path)))
}

/// Returns the brotli sidecar of `path` when the bundle contains one.
pub fn brotli(path: &str) -> Option<Vec<u8>> {
  WebAssets::get(&format!("{path}.br")).map(|file| file.data.into_owned())
}

/// Resolves `path` from the bundle, preferring a precompressed variant when the
/// client accepts brotli. The returned content type always describes the
/// decoded file, not the `.br` sidecar.
pub fn resolve(path: &str, accept_brotli: bool) -> Option<Asset> {
  if accept_brotli && let Some(body) = brotli(path) {
    return Some(Asset {
      body,
      content_type: content_type(path),
      encoding: Some("br"),
    });
  }
  get(path).map(|(body, content_type)| Asset {
    body,
    content_type,
    encoding: None,
  })
}

/// Client-side routes carry no file extension; everything else is a file.
pub fn is_spa_route(path: &str) -> bool {
  let trimmed = path.trim_start_matches('/');
  let last = trimmed.rsplit('/').next().unwrap_or(trimmed);
  if last.is_empty() {
    return true;
  }
  std::path::Path::new(last).extension().is_none()
}

/// Mapped explicitly so serving never depends on the host's MIME database.
pub fn content_type(path: &str) -> &'static str {
  let file = path.rsplit('/').next().unwrap_or(path);
  let Some((_, extension)) = file.rsplit_once('.') else {
    return "application/octet-stream";
  };
  match extension.to_ascii_lowercase().as_str() {
    "html" | "htm" => "text/html; charset=utf-8",
    "css" => "text/css; charset=utf-8",
    "js" | "mjs" => "text/javascript; charset=utf-8",
    "json" | "map" => "application/json",
    "webmanifest" => "application/manifest+json",
    "svg" => "image/svg+xml",
    "png" => "image/png",
    "jpg" | "jpeg" => "image/jpeg",
    "gif" => "image/gif",
    "webp" => "image/webp",
    "avif" => "image/avif",
    "ico" => "image/x-icon",
    "woff" => "font/woff",
    "woff2" => "font/woff2",
    "ttf" => "font/ttf",
    "otf" => "font/otf",
    "wasm" => "application/wasm",
    "txt" => "text/plain; charset=utf-8",
    "xml" => "application/xml",
    "pdf" => "application/pdf",
    _ => "application/octet-stream",
  }
}
