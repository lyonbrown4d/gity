use super::job_schemas::Job;
use crate::openapi::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, ToSchema)]
#[schema(as=CiRunner)]
pub struct Runner {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: String,
    pub tags: String,
    pub status: String,
    pub active: bool,
    pub last_contact_at: String,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}
#[derive(Deserialize, ToSchema)]
#[schema(as=CiRunnerRegister)]
pub struct Register {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<String>,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiRunnerRegistration)]
pub struct Registration {
    pub runner: Runner,
    /// Returned only on registration; store it in the runner secret configuration.
    pub token: String,
}
#[derive(Deserialize, ToSchema)]
#[schema(as=CiRunnerToken)]
pub struct Token {
    /// Project-scoped runner registration token.
    #[schema(write_only)]
    pub token: String,
}
#[derive(Deserialize, ToSchema)]
#[schema(as=CiRunnerClaim)]
pub struct Claim {
    #[serde(flatten)]
    pub auth: Token,
    #[schema(minimum = 30, maximum = 3600, default = 180)]
    pub lease_seconds: Option<i64>,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiRunnerClaimed)]
pub struct Claimed {
    pub claimed: bool,
    pub runner: Runner,
    pub job: Option<Job>,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiRunnerHeartbeat)]
pub struct Heartbeat {
    #[serde(flatten)]
    pub runner: Runner,
    /// Running leases still owned by this runner; missing IDs must be canceled locally.
    pub running_job_ids: Vec<String>,
    /// Current attempt for each active lease; older processes must stop execution.
    pub running_job_attempts: std::collections::BTreeMap<String, i64>,
}
#[derive(Deserialize, ToSchema)]
#[schema(as=CiRunnerAttempt)]
pub struct Attempt {
    #[serde(flatten)]
    pub auth: Token,
    /// Required on retried jobs to reject delayed requests from an older attempt.
    pub attempt: Option<i64>,
}
#[derive(Deserialize, ToSchema)]
#[schema(as=CiRunnerFinish)]
pub struct Finish {
    #[serde(flatten)]
    pub lease: Attempt,
    pub result: Option<String>,
    pub error: Option<String>,
    pub exit_code: Option<i64>,
}
#[derive(Deserialize, ToSchema)]
#[schema(as=CiRunnerAppendTrace)]
pub struct AppendTrace {
    #[serde(flatten)]
    pub lease: Attempt,
    #[schema(max_length = 1048576)]
    pub output: String,
    pub output_truncated: Option<bool>,
    pub duration_millis: Option<i64>,
    /// Flush a held suffix at the end of the trace, preserving masked secret safety.
    #[serde(rename = "final")]
    pub final_chunk: Option<bool>,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiRunnerAppended)]
pub struct Appended {
    pub appended: bool,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiRunnerSourceArchive)]
pub struct SourceArchive {
    pub file_name: String,
    pub encoding: String,
    pub content_base64: String,
}
#[derive(Deserialize, ToSchema)]
#[schema(as=CiRunnerUploadArtifact)]
pub struct UploadArtifact {
    #[serde(flatten)]
    pub lease: Attempt,
    pub name: Option<String>,
    pub file_name: String,
    pub file_path: Option<String>,
    pub content_type: Option<String>,
    /// Base64-encoded file, limited to 64 MiB after decoding.
    pub content_base64: String,
}
