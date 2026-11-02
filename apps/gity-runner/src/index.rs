//! In-process execution of `kind=index` jobs.
//!
//! No container is involved: the runner downloads the pinned source archive,
//! scans it with `gity-code-intel`, and uploads the symbol archive as a bounded
//! job artifact for the platform to ingest.
use crate::client::Client;
use crate::executor::Outcome;
use crate::{heartbeat, output, source};
use anyhow::{Context, Result, bail};
use gity_code_intel::{Archive, archive, scan};
use serde::Deserialize;
use serde_json::json;
use std::{
  path::{Path, PathBuf},
  time::Instant,
};

/// Fallback when an older payload does not name the artifact.
pub(crate) const ARTIFACT_NAME: &str = "code-index.msgpack.zst";
const MAX_ARTIFACT_BYTES: usize = 64 * 1024 * 1024;
const MAX_SOURCE_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Clone, Deserialize)]
pub struct Job {
  #[serde(default)]
  pub commit_sha: String,
  #[serde(default)]
  pub artifact_name: String,
}

struct Scan {
  files: usize,
  definitions: usize,
  bytes: Vec<u8>,
}

pub async fn execute(
  client: &Client,
  id: &str,
  attempt: i64,
  job: Job,
  work_root: &Path,
) -> Result<Outcome> {
  if job.commit_sha.is_empty() {
    bail!("code index job requires a commit SHA");
  }
  let artifact_name = if job.artifact_name.is_empty() {
    ARTIFACT_NAME.to_owned()
  } else {
    job.artifact_name.clone()
  };
  check_name(&artifact_name)?;
  let started = Instant::now();
  let temporary = tempfile::Builder::new()
    .prefix("gity-index-")
    .tempdir_in(work_root)?;
  let workspace = temporary.path().join("source");
  tokio::fs::create_dir(&workspace).await?;
  let compressed = temporary.path().join("source.zip");
  client
    .download(
      &format!("/runners/jobs/{id}/source"),
      json!({"attempt":attempt}),
      &compressed,
      MAX_SOURCE_BYTES,
    )
    .await?;
  let commit = job.commit_sha;
  let mut pulse = heartbeat::spawn(client, id.to_owned(), attempt);
  let task = tokio::spawn(async move { scan_source(compressed, workspace, commit).await });
  let scanned: Result<Scan> = tokio::select! {
    _=crate::shutdown::cancelled()=>Err(anyhow::anyhow!("Runner is shutting down")),
    result=&mut pulse=>match result {
      Ok(Err(error))=>Err(error),
      Ok(Ok(()))=>Err(anyhow::anyhow!("CI job heartbeat stopped")),
      Err(error)=>Err(error.into()),
    },
    result=task=>match result {
      Ok(scanned)=>scanned,
      Err(error)=>Err(anyhow::anyhow!("code index scan task failed: {error}")),
    },
  };
  pulse.abort();
  let scanned = scanned?;
  if scanned.bytes.len() > MAX_ARTIFACT_BYTES {
    bail!(
      "code index archive is {} bytes for {} files; job artifacts are limited to 64 MiB",
      scanned.bytes.len(),
      scanned.files
    );
  }
  output::upload_artifact(
    client,
    id,
    attempt,
    &artifact_name,
    &artifact_name,
    "application/octet-stream",
    &scanned.bytes,
  )
  .await?;
  Ok(Outcome {
    success: true,
    exit_code: 0,
    duration_millis: started.elapsed().as_millis(),
    detail: format!(
      "files={} definitions={} archive_bytes={}",
      scanned.files,
      scanned.definitions,
      scanned.bytes.len()
    ),
  })
}

async fn scan_source(compressed: PathBuf, workspace: PathBuf, commit: String) -> Result<Scan> {
  tokio::task::spawn_blocking(move || {
    source::extract_reader(std::fs::File::open(compressed)?, &workspace)?;
    let files = scan::directory(&workspace, scan::Limits::default())?;
    let archive = Archive::new(commit, files);
    let definitions = archive.definition_count();
    let files = archive.files.len();
    Ok::<Scan, anyhow::Error>(Scan {
      files,
      definitions,
      bytes: archive::encode(&archive)?,
    })
  })
  .await
  .context("code index worker panicked")?
}

fn check_name(name: &str) -> Result<()> {
  if name.is_empty()
    || name == "."
    || name == ".."
    || name.contains(['/', '\\', '\0'])
    || Path::new(name)
      .components()
      .any(|part| !matches!(part, std::path::Component::Normal(_)))
  {
    bail!("invalid code index artifact name");
  }
  Ok(())
}

#[cfg(test)]
#[path = "index_tests.rs"]
mod tests;
