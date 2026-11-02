//! Decides when a push needs a code index job and enqueues it.
use crate::AppState;
use gity_ci::NewJob;
use gity_core::AppError;
use serde_json::{Value, json};

/// Job kind accepted by runner claims and lease scanning, never by the public
/// create endpoint.
pub(crate) const JOB_KIND: &str = "index";
/// Only a runner that registered this tag receives index jobs.
pub(crate) const RUNNER_TAG: &str = "gity-indexer";
/// Archive name the runner uploads and the platform ingests.
pub(crate) const ARTIFACT_NAME: &str = "code-index.msgpack.zst";

/// Enqueues one index job for a default-branch push that contains a supported
/// language. Returns `Ok(None)` when the push is not eligible.
pub(crate) async fn enqueue(
  s: &AppState,
  project: &Value,
  reference: &str,
  commit: &str,
) -> Result<Option<i64>, AppError> {
  if commit.is_empty() || commit.bytes().all(|byte| byte == b'0') {
    return Ok(None);
  }
  if !reference.starts_with("refs/heads/") && reference.starts_with("refs/") {
    return Ok(None);
  }
  let branch = reference.strip_prefix("refs/heads/").unwrap_or(reference);
  let project_id = super::project_id(project)?;
  if branch != crate::repository::default_branch(project) {
    return Ok(None);
  }
  let repo = crate::repository::repo(project);
  let languages = s
    .git
    .analyze_languages(&repo, commit, branch)
    .await
    .map_err(crate::repository::git_error)?;
  if !supported(&languages) {
    return Ok(None);
  }
  let row = s
    .ci
    .create_system_job(NewJob {
      project_id,
      kind: JOB_KIND.into(),
      payload: payload(project, branch, commit).to_string(),
      // A retry only repeats a failed transport; indexing itself is idempotent.
      max_attempts: 2,
      run_after: chrono::Utc::now().naive_utc(),
    })
    .await?;
  tracing::info!(
    project_id,
    job_id = row.id,
    branch,
    commit,
    "code index job enqueued"
  );
  Ok(Some(row.id))
}

fn payload(project: &Value, branch: &str, commit: &str) -> Value {
  json!({
    "project_full_path": project["full_path"],
    "ref_name": branch,
    "commit_sha": commit,
    "tags": [RUNNER_TAG],
    "artifact_name": ARTIFACT_NAME,
    "pipeline_job_name": "code-index",
    "trusted_config": false
  })
}

/// Keep in sync with `gity_code_intel::Language::from_path`, which currently
/// recognises Rust sources only.
fn supported(languages: &Value) -> bool {
  languages["languages"]
    .as_array()
    .is_some_and(|rows| rows.iter().any(|row| row["language"] == "Rust"))
}
