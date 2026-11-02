use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ReleaseLinkCreate {
  pub project_release_id: i64,
  pub name: String,
  pub url: String,
  pub link_type: String,
}
