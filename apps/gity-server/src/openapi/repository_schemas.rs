use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct BranchPath {
    pub id: i64,
    pub branch: String,
}
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in=Query)]
pub struct RepositoryQuery {
    pub ref_name: Option<String>,
    #[serde(rename = "ref")]
    pub revision_alias: Option<String>,
    pub branch_name: Option<String>,
    pub path: Option<String>,
    #[param(minimum = 1, maximum = 500)]
    pub limit: Option<usize>,
    pub query: Option<String>,
    pub q: Option<String>,
    pub match_case: Option<bool>,
    pub regex: Option<bool>,
}
#[derive(Serialize, ToSchema)]
pub struct Ack {
    pub ok: bool,
}
#[derive(Serialize, ToSchema)]
pub struct BranchProtection {
    pub id: String,
    pub project_id: String,
    pub branch_name: String,
    pub rule_type: String,
    pub push_access_level: String,
    pub merge_access_level: String,
    pub require_merge_request: bool,
    pub require_pipeline_success: bool,
    pub allow_force_push: bool,
    pub allow_delete: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Deserialize, ToSchema)]
pub struct ProtectionInput {
    pub push_access_level: Option<String>,
    pub merge_access_level: Option<String>,
    pub require_merge_request: Option<bool>,
    pub require_pipeline_success: Option<bool>,
    pub allow_force_push: Option<bool>,
    pub allow_delete: Option<bool>,
}
#[derive(Serialize, ToSchema)]
pub struct Branch {
    pub name: String,
    pub hash: String,
    pub is_default: bool,
    pub protected: bool,
    pub repository_id: String,
    pub is_protected: bool,
    pub last_commit_sha: String,
    pub protection: Option<BranchProtection>,
}
#[derive(Serialize, ToSchema)]
pub struct CreatedBranch {
    pub name: String,
    pub repository_id: String,
    pub last_commit_sha: String,
    pub is_protected: bool,
    pub protection: Option<BranchProtection>,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateBranch {
    pub name: Option<String>,
    pub branch_name: Option<String>,
    pub source_ref: Option<String>,
    #[serde(rename = "ref")]
    pub source_alias: Option<String>,
}
#[derive(Deserialize, ToSchema)]
pub struct FileCommitInput {
    pub branch_name: Option<String>,
    pub file_path: Option<String>,
    pub path: Option<String>,
    pub content: String,
    pub message: String,
}
#[derive(Serialize, ToSchema)]
pub struct FileCommit {
    pub repository_id: String,
    pub branch_name: String,
    pub file_path: String,
    pub path: String,
    pub message: String,
    pub commit_sha: String,
}
#[derive(Serialize, ToSchema)]
pub struct TreeEntry {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub mode: String,
    pub size: u64,
    pub kind: String,
    pub oid: String,
}
#[derive(Serialize, ToSchema)]
pub struct Blob {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub encoding: String,
    pub content: String,
    pub is_binary: bool,
}
#[derive(Serialize, ToSchema)]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub author_name: String,
    pub author_email: String,
    pub message: String,
    pub committed_at: String,
    pub repository_id: String,
    pub branch_name: String,
    pub commit_sha: String,
    pub author_user_id: String,
    pub created_at: String,
}
#[derive(Serialize, ToSchema)]
pub struct Language {
    pub language: String,
    pub bytes: u64,
    pub percentage: f64,
}
#[derive(Serialize, ToSchema)]
pub struct Languages {
    pub revision: String,
    pub total_bytes: u64,
    pub languages: Vec<Language>,
}
#[derive(Serialize, ToSchema)]
pub struct SearchMatch {
    pub path: String,
    pub line_number: u64,
    pub column: u64,
    pub match_length: u64,
    pub line_content: String,
}
#[derive(Serialize, ToSchema)]
pub struct IndexStatus {
    pub revision: String,
    pub indexed_files: u64,
    pub total_bytes: u64,
    pub truncated: bool,
    pub unchanged: bool,
}
