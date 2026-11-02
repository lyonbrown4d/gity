//! Offline Docker Compose snapshots. Bundles contain credentials and must stay private.
use anyhow::{Context, Result, bail};
use clap::Args;
use std::path::{Path, PathBuf};

mod capture;
mod docker;
#[cfg(test)]
mod metadata_tests;
mod model;
mod ownership;
mod restore;
#[cfg(test)]
mod tests;
mod verify;
pub use model::Manifest;

#[derive(Clone, Debug, Args)]
pub struct BackupArgs {
  /// Exact existing Compose project to stop and snapshot.
  #[arg(long)]
  pub project: String,
  #[arg(long)]
  pub compose_file: PathBuf,
  #[arg(long)]
  pub env_file: Option<PathBuf>,
  /// New directory for the private bundle; must not already exist.
  #[arg(long)]
  pub output: PathBuf,
  #[arg(long, default_value = "debian:bookworm-slim")]
  pub helper_image: String,
  /// Restart only the containers that were running before the snapshot.
  #[arg(long)]
  pub resume_source: bool,
}

#[derive(Clone, Debug, Args)]
pub struct RestoreArgs {
  #[arg(long)]
  pub backup: PathBuf,
  /// New project name. Existing project containers, networks or volumes are rejected.
  #[arg(long)]
  pub project: String,
  /// New private directory for restored bind mounts and resolved Compose config.
  #[arg(long)]
  pub secrets_dir: PathBuf,
  #[arg(long, default_value = "debian:bookworm-slim")]
  pub helper_image: String,
}

pub struct RestoreResult {
  pub compose_file: PathBuf,
  pub project: String,
}

/// Take an offline consistent cut of every volume/bind mount in an exact Compose project.
/// On failure after stopping, the source stays stopped, including with `resume_source`.
/// Coordinate operators and external writers before invocation; no hot backup is provided.
pub fn backup(root: &Path, args: BackupArgs) -> Result<Manifest> {
  model::validate_project(&args.project)?;
  if args.output.exists() {
    bail!("backup output must be a new directory");
  }
  docker::verify_helper(&args.helper_image)?;
  let mut config = docker::compose_config(root, &args)?;
  let containers = docker::project_containers(&args.project)?;
  if containers.is_empty() {
    bail!("source Compose project has no containers");
  }
  capture::apply_container_identity(&mut config, &containers)?;
  let sources = capture::sources(&config, &args.project, &containers)?;
  let running = docker::running_ids(&containers);
  docker::stop_project(&args.project, &running)?;
  let stopped = docker::project_containers(&args.project)?;
  let ids = |items: &[serde_json::Value]| -> std::collections::BTreeSet<String> {
    items
      .iter()
      .filter_map(|c| c["Id"].as_str().map(str::to_owned))
      .collect()
  };
  if ids(&stopped) != ids(&containers) || !docker::running_ids(&stopped).is_empty() {
    bail!("project membership changed or a container is still running; source left stopped");
  }
  model::create_private_dir(&args.output)?;
  let output = args.output.canonicalize()?;
  let manifest = capture::write_bundle(&args, &output, config, sources)?;
  if let Err(error) = docker::assert_stopped(&args.project) {
    // Never leave a completed manifest when the final offline check fails.
    std::fs::remove_file(output.join("manifest.json"))?;
    return Err(error);
  }
  if args.resume_source && !running.is_empty() {
    docker::status(
      docker::command().arg("start").args(&running),
      "resume source containers",
    )?;
  }
  println!(
    "Offline backup: {} (source resumed: {})",
    output.display(),
    args.resume_source
  );
  Ok(manifest)
}

/// Validate the complete bundle before allocating new resources. Never starts services.
/// On failure, newly created volumes and the new private asset directory are removed.
pub fn restore(_root: &Path, args: RestoreArgs) -> Result<RestoreResult> {
  model::validate_project(&args.project)?;
  let bundle = args.backup.canonicalize().context("locate backup bundle")?;
  let (manifest, config) = verify::bundle(&bundle)?;
  docker::verify_helper(&args.helper_image)?;
  if manifest.source_project == args.project {
    bail!("restore project must differ from source project");
  }
  docker::ensure_empty_project(&args.project)?;
  if args.secrets_dir.exists() {
    bail!("restore secrets directory must not exist");
  }
  restore::run(&args, &bundle, &manifest, config)
}
