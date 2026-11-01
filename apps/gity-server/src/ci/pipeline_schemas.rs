use super::{config::PipelineConfig, job_schemas::Job};
use crate::openapi::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(as=CiPipelineStatus)]
pub enum Status {
    Pending,
    Running,
    Succeeded,
    Failed,
    Canceled,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiPipeline)]
pub struct Pipeline {
    pub id: String,
    pub project_id: String,
    pub iid: i64,
    pub name: String,
    pub source: String,
    pub ref_name: String,
    pub commit_sha: String,
    pub status: Status,
    /// `.gity-ci.yml` for repository configuration; `inline` for an API override.
    pub config_source: String,
    pub config_content: String,
    #[serde(flatten)]
    pub timestamps: Timestamps,
    pub started_at: String,
    pub finished_at: String,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiPipelineDetail)]
pub struct Detail {
    pub pipeline: Pipeline,
    pub spec: PipelineConfig,
    pub jobs: Vec<PipelineJob>,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiPipelineJobLink)]
pub struct JobLink {
    pub id: String,
    pub project_id: String,
    pub pipeline_id: String,
    pub project_job_id: String,
    pub name: String,
    pub stage: String,
    pub image: String,
    /// JSON encoded dependency names, retained for API compatibility.
    pub needs: String,
    pub script: String,
    pub artifacts: String,
    pub sort_order: i64,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiPipelineJob)]
pub struct PipelineJob {
    pub pipeline_job: JobLink,
    pub project_job: Job,
    /// `blocked` means a pending job awaits its dependencies.
    pub status: String,
    pub needs: Vec<String>,
    pub script: Vec<String>,
    pub artifacts: Vec<String>,
    pub tags: Vec<String>,
}
#[derive(Deserialize, ToSchema)]
#[schema(as=CiPipelineLint)]
pub struct Lint {
    pub config_content: String,
}
#[derive(Serialize, ToSchema)]
#[schema(as=CiPipelineLintResult)]
pub struct LintResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub spec: Option<PipelineConfig>,
}
