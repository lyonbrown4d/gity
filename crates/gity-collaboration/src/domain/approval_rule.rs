use super::Change;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ApprovalRuleCreate {
  pub project_id: i64,
  pub name: String,
  pub target_branch: String,
  pub approvals_required: i32,
  pub eligible_user_ids: String,
  pub code_owner: i32,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ApprovalRuleChanges {
  pub name: Change<String>,
  pub target_branch: Change<String>,
  pub approvals_required: Change<i32>,
  pub eligible_user_ids: Change<String>,
  pub code_owner: Change<i32>,
}
