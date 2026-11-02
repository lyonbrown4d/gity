use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct IssueCommentCreate {
  pub project_issue_id: i64,
  pub author_user_id: i64,
  pub body: String,
}
