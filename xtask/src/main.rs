//! Repository tasks intentionally require only Rust tools and native Git.
use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, process::Command};
mod blackbox;
mod container_stats;
mod docs;
mod docs_mermaid;
mod git_stress;
mod k6;
mod precompress;
mod sources;
#[derive(Parser)]
#[command(about = "Gity Cargo build and verification tasks")]
struct Cli {
  #[command(subcommand)]
  command: Option<Task>,
}
#[derive(Subcommand)]
enum Task {
  Check,
  Test,
  Web,
  Release,
  VendorCheck,
  SourceCheck,
  /// Precompress static Web assets with brotli for `ServeDir::precompressed_br`.
  Precompress,
  /// Build or preview the mdBook manual.
  Docs {
    #[arg(long)]
    serve: bool,
  },
  /// Internal mdBook preprocessor endpoint; mdBook invokes it, not users.
  #[command(hide = true)]
  DocsPreprocess {
    /// Arguments mdBook appends, such as `supports html`.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
  },
  GitStress(git_stress::GitStressArgs),
  K6(k6::K6Args),
  ContainerStats(container_stats::ContainerStatsArgs),
  Blackbox(blackbox::BlackboxArgs),
}
#[derive(serde::Deserialize)]
struct Manifest {
  files: Vec<Artifact>,
}
#[derive(serde::Deserialize)]
struct Artifact {
  path: String,
  sha256: String,
}
fn main() -> Result<()> {
  let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .context("workspace root")?
    .to_path_buf();
  std::env::set_current_dir(&root)?;
  match Cli::parse().command.unwrap_or(Task::Check) {
    Task::Docs { serve } => docs::run(&root, serve)?,
    Task::DocsPreprocess { args } => docs_mermaid::run(&args)?,
    Task::VendorCheck => verify_vendor(&root)?,
    Task::SourceCheck => sources::check(&root)?,
    Task::Precompress => precompress::run(&root)?,
    Task::GitStress(args) => git_stress::run(&root, args)?,
    Task::K6(args) => k6::run(&root, args)?,
    Task::ContainerStats(args) => container_stats::run(args)?,
    Task::Blackbox(args) => blackbox::run(&root, args)?,
    Task::Check => {
      sources::check(&root)?;
      verify_vendor(&root)?;
      run("cargo", &["fmt", "--all", "--", "--check"])?;
      run("cargo", &["check", "--workspace", "--locked"])?;
      run(
        "cargo",
        &[
          "check",
          "-p",
          "gity-web",
          "--target",
          "wasm32-unknown-unknown",
          "--locked",
        ],
      )?;
    }
    Task::Test => {
      verify_vendor(&root)?;
      run("cargo", &["test", "--workspace", "--locked"])?;
    }
    task @ (Task::Web | Task::Release) => {
      verify_vendor(&root)?;
      let release = matches!(task, Task::Release);
      let mut command = Command::new("trunk");
      command
        .current_dir(root.join("apps/gity-web"))
        .env("NO_COLOR", "true")
        .arg("build");
      if release {
        command.arg("--release");
      }
      if !command
        .status()
        .context("install Trunk with `cargo install trunk --locked`")?
        .success()
      {
        bail!("Web build failed");
      }
      precompress::run(&root)?;
      if release {
        // Build each role separately so Git-only does not inherit full-server features.
        for package in [
          "gity-server",
          "gity-runner",
          "gity-git-hook",
          "gity-search-index",
          "gity-standalone",
          "gity-api",
          "gity-git-server",
          "gity-git-node-app",
          "gity-worker",
          "gity-registry",
          "gity-gateway-app",
        ] {
          run("cargo", &["build", "--release", "--locked", "-p", package])?;
        }
      }
    }
  }
  Ok(())
}
fn verify_vendor(root: &std::path::Path) -> Result<()> {
  let directory = root.join("apps/gity-web/vendor");
  let manifest: Manifest =
    toml::from_str(&std::fs::read_to_string(directory.join("manifest.toml"))?)?;
  let mut expected = std::collections::BTreeSet::new();
  for artifact in manifest.files {
    if artifact.path.contains('/') || artifact.path.contains('\\') || artifact.path.starts_with('.')
    {
      bail!("invalid vendor manifest path");
    }
    let digest = hex::encode(Sha256::digest(
      std::fs::read(directory.join(&artifact.path))
        .with_context(|| format!("missing vendor asset {}", artifact.path))?,
    ));
    if digest != artifact.sha256 {
      bail!("vendor asset checksum mismatch: {}", artifact.path);
    }
    expected.insert(artifact.path);
  }
  for entry in std::fs::read_dir(&directory)? {
    let entry = entry?;
    let name = entry.file_name().to_string_lossy().to_string();
    if entry.file_type()?.is_file()
      && (name.ends_with(".js") || name.ends_with(".mjs") || name.ends_with(".css"))
      && !expected.contains(&name)
    {
      bail!("untracked vendor asset: {name}");
    }
  }
  println!("Verified {} third-party editor assets", expected.len());
  Ok(())
}
fn run(program: &str, args: &[&str]) -> Result<()> {
  if !Command::new(program).args(args).status()?.success() {
    bail!("{program} {} failed", args.join(" "));
  }
  Ok(())
}
