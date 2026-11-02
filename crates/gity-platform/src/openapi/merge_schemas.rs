use super::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, ToSchema)]
pub struct MergeRequest {
  pub id: String,
  pub project_id: String,
  pub iid: i64,
  pub author_user_id: String,
  pub title: String,
  pub description: String,
  pub state: String,
  pub source_branch: String,
  pub target_branch: String,
  /// Milestone association serialized as a string identifier.
  pub milestone_id: Option<String>,
  pub merged_at: Option<String>,
  pub merged_by_user_id: Option<String>,
  pub merge_commit_sha: Option<String>,
  /// Pending durable merge operation; null when no reconciliation is required.
  pub active_merge_operation_id: Option<String>,
  /// Source branch commit captured when the merge request was created.
  pub head_sha: Option<String>,
  pub closed_at: Option<String>,
  pub squash: bool,
  pub draft: bool,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateMergeRequest {
  pub title: String,
  pub description: Option<String>,
  pub source_branch: String,
  pub target_branch: String,
  pub author_user_id: Option<i64>,
  pub milestone_id: Option<i64>,
  /// Squash the source commits into one commit on merge.
  pub squash: Option<bool>,
  /// Draft merge requests are not ready for review.
  pub draft: Option<bool>,
}
#[derive(Deserialize, ToSchema)]
pub struct UpdateMergeRequest {
  pub title: Option<String>,
  pub description: Option<String>,
  pub state: Option<String>,
  /// Omit to keep the milestone; send null to detach it.
  pub milestone_id: Option<i64>,
  pub squash: Option<bool>,
  pub draft: Option<bool>,
}
#[derive(Deserialize, ToSchema)]
pub struct MergeCommand {
  pub message: Option<String>,
  pub author_name: Option<String>,
  pub author_email: Option<String>,
}
#[derive(Serialize, ToSchema)]
pub struct MergeDiff {
  pub merge_request: MergeRequest,
  pub diff: String,
}
#[derive(Serialize, ToSchema)]
pub struct Participant {
  pub id: String,
  pub merge_request_id: String,
  pub user_id: String,
  pub role: String,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct Participants {
  pub merge_request: MergeRequest,
  pub participants: Vec<Participant>,
}
#[derive(Serialize, ToSchema)]
pub struct MergeComment {
  pub id: String,
  pub merge_request_id: String,
  pub author_user_id: String,
  pub body: String,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct MergeComments {
  pub merge_request: MergeRequest,
  pub comments: Vec<MergeComment>,
}
#[derive(Serialize, ToSchema)]
pub struct Approval {
  pub id: String,
  pub merge_request_id: String,
  pub user_id: String,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct Approvals {
  pub merge_request: MergeRequest,
  pub approvals: Vec<Approval>,
}
#[derive(Serialize, ToSchema)]
pub struct ApprovalRule {
  pub id: String,
  pub project_id: String,
  pub name: String,
  pub target_branch: String,
  pub approvals_required: i64,
  pub eligible_user_ids: Vec<String>,
  pub code_owner: bool,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct ApprovalRules {
  pub project_id: String,
  pub rules: Vec<ApprovalRule>,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateApprovalRule {
  pub name: String,
  pub target_branch: Option<String>,
  pub approvals_required: Option<i64>,
  pub eligible_user_ids: Option<Vec<i64>>,
  pub code_owner: Option<bool>,
}
#[derive(Deserialize, ToSchema)]
pub struct UpdateApprovalRule {
  pub name: Option<String>,
  pub target_branch: Option<String>,
  pub approvals_required: Option<i64>,
  pub eligible_user_ids: Option<Vec<i64>>,
  pub code_owner: Option<bool>,
}
#[derive(Serialize, ToSchema)]
pub struct ApprovalRuleCheck {
  pub rule_id: String,
  pub name: String,
  pub target_branch: String,
  pub approvals_required: i64,
  pub approval_count: i64,
  pub eligible_user_ids: Vec<String>,
  pub code_owner: bool,
  pub satisfied: bool,
  pub blocking_reason: String,
}
#[derive(Serialize, ToSchema)]
pub struct CheckBlocker {
  pub code: String,
  pub category: String,
  pub message: String,
}
#[derive(Serialize, ToSchema)]
pub struct MergeChecks {
  pub merge_request: MergeRequest,
  pub source_branch: String,
  pub source_commit_sha: String,
  pub target_branch: String,
  pub target_branch_protected: bool,
  pub require_merge_request: bool,
  pub require_pipeline_success: bool,
  pub require_approval: bool,
  pub required_approvals: i64,
  pub approval_count: i64,
  pub approval_rules: Vec<ApprovalRuleCheck>,
  pub merge_access_level: String,
  pub pipeline_required: bool,
  pub required: bool,
  pub mergeable: bool,
  pub status: String,
  pub blocking_reason: String,
  pub blockers: Vec<CheckBlocker>,
  pub pipeline: Option<crate::ci::pipeline_schemas::Pipeline>,
}
