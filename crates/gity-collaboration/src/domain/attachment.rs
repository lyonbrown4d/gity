use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct AttachmentCreate {
  pub id: i64,
  pub project_issue_id: i64,
  pub uploaded_by_user_id: i64,
  pub file_name: String,
  pub content_type: Option<String>,
  pub byte_size: i64,
  pub storage_key: String,
}
