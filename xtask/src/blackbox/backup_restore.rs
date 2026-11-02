use super::{BlackboxArgs, Reporter, RunConfig, api, process, runtime as deployment};
use crate::backup::{self, BackupArgs, RestoreArgs};
use anyhow::Result;
use reqwest::blocking::Client;
use std::{path::Path, process::Command};

mod engine;
mod fixture;
mod probes;
mod restored_ci;
mod runtime;
use fixture::{seed, verify};
use runtime::{Restored, control};

pub(super) fn run(
  root: &Path,
  args: &BlackboxArgs,
  config: &RunConfig,
  scenario: &api::Scenario,
  work: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let client = Client::builder()
    .timeout(std::time::Duration::from_secs(120))
    .build()?;
  let fixture = seed(&client, config, scenario, work)?;
  let public_objects = probes::PublicObjects::seed(&client, config, scenario)?;
  let hosted_objects = engine::HostedObjects::seed(root, args, scenario)?;
  let resolved = work.join("backup-source-compose.json");
  let mut command = Command::new("docker");
  deployment::configure_compose(&mut command, root, args)?;
  command.args(["--profile", "*", "config", "--format", "json"]);
  private_write(
    &resolved,
    &process::run_output(&mut command, "resolve backup source configuration")?.stdout,
  )?;
  let bundle = work.join("offline-backup");
  let manifest = backup::backup(
    root,
    BackupArgs {
      project: args.project.clone(),
      compose_file: resolved,
      env_file: None,
      output: bundle.clone(),
      helper_image: "debian:bookworm-slim".into(),
      resume_source: false,
    },
  )?;
  reporter.pass(
    "offline-backup-complete",
    format!(
      "all project containers stopped; {} volume mappings and {} private bind sources checksummed",
      manifest.volumes.len(),
      manifest.binds.len()
    ),
  );
  let target = format!("{}-restore", args.project);
  let assets = work.join("restored-private-assets");
  let restored = backup::restore(
    root,
    RestoreArgs {
      backup: bundle,
      project: target.clone(),
      secrets_dir: assets.clone(),
      helper_image: "debian:bookworm-slim".into(),
    },
  )?;
  let target_volumes = manifest
    .volumes
    .iter()
    .map(|v| format!("{}_{}", restored.project, v.key))
    .collect();
  let guard = Restored::new(target, restored.compose_file, assets, target_volumes);
  control(&guard, "up")?;
  deployment::wait_for_health(config, reporter)?;
  verify(&client, config, scenario, work, &fixture, "restored")?;
  public_objects.verify(&client, scenario)?;
  hosted_objects.verify(root, args, &guard)?;
  report_object_probes(
    reporter,
    &public_objects,
    &hosted_objects,
    "offline-restore-object-bytes",
  );
  reporter.pass("offline-restore-roundtrip", "new project preserved issue/MR records, attachment/LFS bytes, strict-pinned SSH host identity and exact Git refs; outsider reads and unknown SSH key denied");
  control(&guard, "restart")?;
  deployment::wait_for_health(config, reporter)?;
  verify(
    &client,
    config,
    scenario,
    work,
    &fixture,
    "restored-restart",
  )?;
  public_objects.verify(&client, scenario)?;
  hosted_objects.verify(root, args, &guard)?;
  report_object_probes(
    reporter,
    &public_objects,
    &hosted_objects,
    "offline-restore-restart-object-bytes",
  );
  reporter.pass(
    "offline-restore-restart",
    "restored state and auth boundaries survived service restart",
  );
  restored_ci::run(&client, config, scenario, work, reporter)?;
  guard.finish()?;
  reporter.pass(
    "offline-restore-cleanup",
    "restored target containers, networks and volumes all absent after explicit cleanup",
  );
  Ok(())
}

fn private_write(path: &Path, bytes: &[u8]) -> Result<()> {
  use std::io::Write;
  let mut options = std::fs::OpenOptions::new();
  options.write(true).create_new(true);
  #[cfg(unix)]
  {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o600);
  }
  let mut file = options.open(path)?;
  file.write_all(bytes)?;
  Ok(())
}

fn report_object_probes(
  reporter: &mut Reporter,
  public: &probes::PublicObjects,
  hosted: &engine::HostedObjects,
  name: &str,
) {
  reporter.pass(name, format!("{} public Generic/mirror manifest/blob reads matched exact pre-backup size/SHA256 using original owner token; outsider denied; {} hosted OCI manifest/blob objects matched through private Angos administrative reads (public project remains mirror)", public.count(), hosted.count()));
}
