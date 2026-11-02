//! Brotli precompression for static web assets.
//!
//! `tower-http`'s `ServeDir::precompressed_br` serves `<file>.br` when the
//! client accepts brotli, so the build writes those siblings next to the
//! assets instead of compressing on every request.
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Files smaller than this gain too little to justify a brotli copy.
const MIN_BYTES: u64 = 1024;

/// Extensions worth compressing; everything else is copied as-is by Trunk.
const COMPRESSIBLE: [&str; 5] = ["wasm", "js", "css", "html", "json"];

pub fn run(root: &Path) -> Result<()> {
  let dist = root.join("dist/web");
  anyhow::ensure!(
    dist.join("index.html").exists(),
    "{}: build the Web bundle with `cargo xtask web` first",
    dist.display()
  );
  precompress(&dist)?;
  Ok(())
}

fn precompress(dist: &Path) -> Result<()> {
  let mut pending = vec![dist.to_path_buf()];
  let mut written = 0usize;
  while let Some(directory) = pending.pop() {
    for entry in fs::read_dir(&directory).with_context(|| directory.display().to_string())? {
      let path = entry?.path();
      if path.is_dir() {
        pending.push(path);
        continue;
      }
      if !compressible(&path) {
        continue;
      }
      let source = fs::metadata(&path)?;
      if source.len() < MIN_BYTES {
        continue;
      }
      let target = sibling(&path);
      if fresh(&target, &source) {
        continue;
      }
      let data = fs::read(&path)?;
      let mut compressed = Vec::with_capacity(data.len() / 3);
      let mut params = brotli::enc::BrotliEncoderParams::default();
      params.quality = 6;
      params.lgwin = 22;
      brotli::BrotliCompress(&mut data.as_slice(), &mut compressed, &params)
        .with_context(|| format!("brotli {}", path.display()))?;
      fs::write(&target, &compressed).with_context(|| target.display().to_string())?;
      written += 1;
    }
  }
  println!(
    "brotli: {written} asset(s) precompressed under {}",
    dist.display()
  );
  Ok(())
}

fn compressible(path: &Path) -> bool {
  path
    .extension()
    .and_then(|extension| extension.to_str())
    .is_some_and(|extension| COMPRESSIBLE.contains(&extension))
}

fn sibling(path: &Path) -> PathBuf {
  let mut name = path.file_name().unwrap_or_default().to_os_string();
  name.push(".br");
  path.with_file_name(name)
}

fn fresh(target: &Path, source: &fs::Metadata) -> bool {
  let Ok(existing) = fs::metadata(target) else {
    return false;
  };
  match (existing.modified(), source.modified()) {
    (Ok(compressed), Ok(original)) => compressed >= original,
    _ => false,
  }
}
