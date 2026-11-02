use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct MergeCommentCreate {
  pub merge_request_id: i64,
  pub author_user_id: i64,
  pub body: String,
  pub discussion_id: Option<i64>,
  pub position: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct MergeCommentUpdate {
  pub body: String,
}
