use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{path::Path, process::Command};

/// The manual is built by this mdBook release; keep it in step with CI.
const MDBOOK_VERSION: &str = "0.5.4";

#[derive(serde::Deserialize)]
struct Assets {
  files: Vec<Asset>,
}

#[derive(serde::Deserialize)]
struct Asset {
  path: String,
  sha256: String,
}

pub fn run(root: &Path, serve: bool) -> Result<()> {
  require_mdbook()?;
  verify_assets(root)?;
  let mut command = Command::new("mdbook");
  command.current_dir(root);
  if serve {
    command.args(["serve", "--hostname", "127.0.0.1"]);
  } else {
    command.arg("build");
  }
  ensure!(command.status()?.success(), "mdBook command failed");
  Ok(())
}

/// The mermaid preprocessor is a workspace package, so only mdBook itself has
/// to exist on `PATH`.
fn require_mdbook() -> Result<()> {
  let install = format!("cargo install mdbook --version {MDBOOK_VERSION} --locked");
  let output = Command::new("mdbook")
    .arg("--version")
    .output()
    .with_context(|| format!("install the documentation tool: {install}"))?;
  let reported = String::from_utf8_lossy(&output.stdout);
  let actual = reported
    .trim()
    .strip_prefix("mdbook")
    .map(str::trim)
    .map(|value| value.trim_start_matches('v'));
  ensure!(
    output.status.success() && actual == Some(MDBOOK_VERSION),
    "use the pinned documentation tool: {install}"
  );
  Ok(())
}

fn verify_assets(root: &Path) -> Result<()> {
  let directory = root.join("docs/shared/vendor");
  let manifest: Assets =
    toml::from_str(&std::fs::read_to_string(directory.join("manifest.toml"))?)?;
  for asset in &manifest.files {
    ensure!(
      !asset.path.contains('/') && !asset.path.contains('\\') && !asset.path.starts_with('.'),
      "invalid documentation asset path"
    );
    let checksum = hex::encode(Sha256::digest(std::fs::read(directory.join(&asset.path))?));
    ensure!(
      checksum == asset.sha256,
      "documentation asset checksum mismatch: {}",
      asset.path
    );
  }
  println!("Verified {} documentation assets", manifest.files.len());
  Ok(())
}
