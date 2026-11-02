use chrono::NaiveDateTime;
use serde_json::Value;

pub const ZERO: &str = "1970-01-02T00:00:00.000Z";
pub const BLOCKED: &str = "2037-12-31T00:00:00.000Z";

pub struct NewJob {
  pub project_id: i64,
  pub kind: String,
  pub payload: String,
  pub max_attempts: i32,
  pub run_after: NaiveDateTime,
}
pub struct NewRunner {
  pub project_id: i64,
  pub name: String,
  pub description: String,
  pub tags: String,
  pub token_hash: String,
}
pub struct NewPipeline {
  pub project_id: i64,
  pub name: String,
  pub source: String,
  pub ref_name: String,
  pub commit_sha: String,
  pub config_source: String,
  pub config_content: String,
  pub jobs: Vec<PipelineJobSpec>,
}
pub struct PipelineJobSpec {
  pub name: String,
  pub stage: String,
  pub image: String,
  pub needs: Vec<String>,
  pub script: Vec<String>,
  pub artifacts: Vec<String>,
  pub payload: Value,
}
pub struct ClaimJob {
  pub job_id: i64,
  pub attempt: i32,
  pub runner_id: i64,
  pub lease_seconds: i64,
  pub payload: String,
}
#[derive(Clone)]
pub struct Lease {
  pub job_id: i64,
  pub attempt: i32,
  pub owner: String,
}
pub struct FinishJob {
  pub lease: Lease,
  pub success: bool,
  pub result: String,
  pub error: String,
  pub exit_code: i32,
}
pub struct AppendTrace {
  pub lease: Lease,
  pub output: String,
  pub masked_values: Vec<String>,
  pub final_chunk: bool,
  pub truncated: bool,
  pub duration_millis: i64,
}
pub struct NewArtifact {
  pub id: i64,
  pub project_id: i64,
  pub job_id: i64,
  pub name: String,
  pub file_name: String,
  pub file_path: String,
  pub content_type: String,
  pub byte_size: i64,
  pub storage_key: String,
}
pub fn pipeline_status<'a>(states: impl IntoIterator<Item = &'a str>) -> &'static str {
  let states = states.into_iter().collect::<Vec<_>>();
  if states.contains(&"failed") {
    "failed"
  } else if states.contains(&"canceled") {
    "canceled"
  } else if !states.is_empty() && states.iter().all(|s| *s == "succeeded") {
    "succeeded"
  } else if states.iter().any(|s| ["running", "succeeded"].contains(s)) {
    "running"
  } else {
    "pending"
  }
}
