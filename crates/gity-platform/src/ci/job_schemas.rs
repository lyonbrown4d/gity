use super::pipeline_schemas::Status;
use crate::openapi::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, ToSchema)]
#[schema(as=CiJob)]
pub struct Job {
  pub id: String,
  pub project_id: String,
  pub kind: Kind,
  pub status: Status,
  /// Serialized execution payload. Project views remove environment secrets.
  /// The authenticated runner claim response includes scoped execution variables.
  pub payload: String,
  pub result: String,
  pub attempts: i64,
  pub max_attempts: i64,
  pub run_after: String,
  pub locked_by: String,
  pub locked_until: String,
  pub last_error: String,
  pub started_at: String,
  pub finished_at: String,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(as=CiJobKind)]
pub enum Kind {
  Noop,
  Script,
}
#[derive(Deserialize, ToSchema)]
#[schema(as=CiJobCreateJob)]
pub struct CreateJob {
  pub kind: Kind,
  /// JSON script payload containing script, image, tags, env, artifacts and timeout_seconds.
  pub payload: String,
  #[schema(minimum = 1, maximum = 10, default = 3)]
  pub max_attempts: Option<i64>,
  #[schema(format=DateTime)]
  pub run_after: Option<String>,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiJobLog)]
pub struct JobLog {
  pub id: Option<String>,
  pub project_id: Option<String>,
  pub project_job_id: Option<String>,
  pub attempt: i64,
  pub exit_code: i64,
  pub output: String,
  pub output_truncated: Option<i64>,
  pub duration_millis: Option<i64>,
  pub created_at: Option<String>,
  pub updated_at: Option<String>,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiJobTrace)]
pub struct Trace {
  pub job: Job,
  pub attempts: Vec<JobLog>,
  pub log: Option<JobLog>,
  pub output: String,
  /// Present in bounded mode; metadata only with empty output fields.
  pub attempt_summaries: Option<Vec<JobLog>>,
  pub attempt: Option<i32>,
  /// Actual byte offset, advanced when the requested offset splits UTF-8.
  pub offset: Option<u64>,
  pub next_offset: Option<u64>,
  pub total_bytes: Option<u64>,
  pub has_more: Option<bool>,
  /// True when persisted output was truncated by runner or storage limits.
  pub truncated: Option<bool>,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiJobArtifact)]
pub struct Artifact {
  pub id: String,
  pub project_id: String,
  pub project_job_id: String,
  pub name: String,
  pub file_name: String,
  pub file_path: String,
  pub content_type: String,
  pub byte_size: i64,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiJobArtifactDownload)]
pub struct ArtifactDownload {
  #[serde(flatten)]
  pub artifact: Artifact,
  pub encoding: String,
  pub content_base64: String,
}
