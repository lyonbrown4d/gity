use super::Change;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct IssueCreate {
  pub project_id: i64,
  pub author_user_id: i64,
  pub title: String,
  pub description: Option<String>,
  pub milestone_id: Option<i64>,
  pub due_date: Option<chrono::NaiveDate>,
  pub weight: Option<i64>,
  pub confidential: i64,
  pub issue_type: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct IssueChanges {
  pub title: Change<String>,
  pub description: Change<Option<String>>,
  pub state: Change<String>,
  pub milestone_id: Change<Option<i64>>,
  pub due_date: Change<Option<chrono::NaiveDate>>,
  pub weight: Change<Option<i64>>,
  pub confidential: Change<i64>,
  pub issue_type: Change<String>,
  pub updated_by_user_id: Change<Option<i64>>,
  pub closed_at: Change<Option<chrono::NaiveDateTime>>,
  pub closed_by_user_id: Change<Option<i64>>,
}
