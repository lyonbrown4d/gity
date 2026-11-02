//! Repository tasks intentionally require only Rust tools and native Git.
use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::{path::PathBuf, process::Command};
mod backup;
mod blackbox;
mod container_stats;
mod docs;
mod docs_mermaid;
mod git_stress;
mod k6;
mod persistence_policy;
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
  Backup(backup::BackupArgs),
  Restore(backup::RestoreArgs),
  Check,
  Test,
  Web,
  Release,
  SourceCheck,
  /// Build the Vite front-end into `dist/web` and precompress its assets.
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
fn main() -> Result<()> {
  let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .context("workspace root")?
    .to_path_buf();
  std::env::set_current_dir(&root)?;
  match Cli::parse().command.unwrap_or(Task::Check) {
    Task::Docs { serve } => docs::run(&root, serve)?,
    Task::DocsPreprocess { args } => docs_mermaid::run(&args)?,
    Task::SourceCheck => sources::check(&root)?,
    Task::Precompress => precompress::run(&root)?,
    Task::GitStress(args) => git_stress::run(&root, args)?,
    Task::K6(args) => k6::run(&root, args)?,
    Task::ContainerStats(args) => container_stats::run(args)?,
    Task::Blackbox(args) => blackbox::run(&root, args)?,
    Task::Backup(args) => {
      backup::backup(&root, args)?;
    }
    Task::Restore(args) => {
      backup::restore(&root, args)?;
    }
    Task::Check => {
      sources::check(&root)?;
      run("cargo", &["fmt", "--all", "--", "--check"])?;
      run("cargo", &["check", "--workspace", "--locked"])?;
    }
    Task::Test => {
      run("cargo", &["test", "--workspace", "--locked"])?;
    }
    task @ (Task::Web | Task::Release) => {
      let release = matches!(task, Task::Release);
      let mut command = Command::new("pnpm");
      command
        .current_dir(root.join("web"))
        .env("NO_COLOR", "true")
        .arg("build");
      if !command
        .status()
        .context("install pnpm and run `pnpm --dir web install`")?
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
fn run(program: &str, args: &[&str]) -> Result<()> {
  if !Command::new(program).args(args).status()?.success() {
    bail!("{program} {} failed", args.join(" "));
  }
  Ok(())
}
