use super::Change;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiPageCreate {
  pub project_id: i64,
  pub slug: String,
  pub title: String,
  pub content: Option<String>,
  pub format: String,
  pub author_user_id: i64,
  pub last_edited_by_user_id: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiPageChanges {
  pub title: Change<String>,
  pub content: Change<Option<String>>,
  pub last_edited_by_user_id: Change<i64>,
}
