use super::Change;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct MergeRequestCreate {
  pub project_id: i64,
  pub author_user_id: i64,
  pub title: String,
  pub description: Option<String>,
  pub source_branch: String,
  pub target_branch: String,
  pub milestone_id: Option<i64>,
  pub head_sha: Option<String>,
  pub squash: i64,
  pub draft: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct MergeRequestChanges {
  pub title: Change<String>,
  pub description: Change<Option<String>>,
  pub state: Change<String>,
  pub milestone_id: Change<Option<i64>>,
  pub squash: Change<i64>,
  pub draft: Change<i64>,
  pub closed_at: Change<Option<chrono::NaiveDateTime>>,
}
