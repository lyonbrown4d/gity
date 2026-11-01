use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{path::Path, process::Command};

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
  for (tool, version) in [("mdbook", "0.5.4"), ("mdbook-mermaid", "0.17.1")] {
    let install = format!("cargo install {tool} --version {version} --locked");
    let output = Command::new(tool)
      .arg("--version")
      .output()
      .with_context(|| format!("install documentation tool: {install}"))?;
    let reported = String::from_utf8_lossy(&output.stdout);
    let actual = reported
      .trim()
      .strip_prefix(tool)
      .map(str::trim)
      .map(|value| value.trim_start_matches('v'));
    ensure!(
      output.status.success() && actual == Some(version),
      "use the pinned documentation tool: {install}"
    );
  }
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
