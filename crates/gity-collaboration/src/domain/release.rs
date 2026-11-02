use super::Change;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ReleaseCreate {
  pub project_id: i64,
  pub tag_name: String,
  pub name: String,
  pub description: String,
  pub created_by_user_id: i64,
  pub released_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ReleaseChanges {
  pub name: Change<String>,
  pub description: Change<String>,
  pub released_at: Change<chrono::NaiveDateTime>,
}
