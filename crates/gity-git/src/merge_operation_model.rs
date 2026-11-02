use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergeOperationInput {
  pub operation_key: String,
  pub actor_user_id: i64,
  pub source_branch: String,
  pub target_branch: String,
  pub source_sha: String,
  pub target_sha: String,
  pub message: String,
  pub author_name: String,
  pub author_email: String,
  pub timestamp: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum MergeOperationOutcome {
  Applied { commit_sha: String },
  Rejected { reason: String, actor_user_id: i64 },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Receipt {
  pub input: MergeOperationInput,
  pub outcome: MergeOperationOutcome,
}
impl MergeOperationInput {
  pub(super) fn marker(&self) -> Result<String> {
    // Canonical UUID v7 avoids path/ref injection without accepting arbitrary
    // object names from the protocol as internal receipt references.
    let value = uuid::Uuid::parse_str(&self.operation_key)?;
    ensure!(
      value.get_version_num() == 7
        && value.get_variant() == uuid::Variant::RFC4122
        && value.to_string() == self.operation_key,
      "invalid merge operation key"
    );
    Ok(format!("refs/gity/merge-operations/{}", self.operation_key))
  }
  pub(super) fn prepared(&self) -> String {
    format!("refs/gity/merge-prepared/{}", self.operation_key)
  }
}
