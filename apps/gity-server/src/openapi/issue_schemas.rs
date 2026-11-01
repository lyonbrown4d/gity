use super::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, ToSchema)]
pub struct Issue {
    /// Project-local issue number serialized as a string.
    pub id: String,
    pub repository_id: String,
    pub number: i64,
    pub title: String,
    pub description: String,
    #[schema(example = "open")]
    pub status: String,
    pub author_user_id: String,
    pub assignee_user_id: Option<String>,
    pub assignee_user_ids: Vec<String>,
    pub assignees: Vec<Assignee>,
    pub labels: Vec<Label>,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateIssue {
    pub title: String,
    pub description: Option<String>,
    pub author_user_id: Option<i64>,
}
#[derive(Deserialize, ToSchema)]
pub struct UpdateIssue {
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub status: Option<String>,
}
#[derive(Serialize, ToSchema)]
pub struct Comment {
    pub id: String,
    pub issue_id: String,
    pub author_user_id: String,
    pub content: String,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateComment {
    pub body: Option<String>,
    pub content: Option<String>,
    pub author_user_id: Option<i64>,
}
#[derive(Serialize, ToSchema)]
pub struct Assignee {
    pub id: String,
    pub issue_id: String,
    pub user_id: String,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct LabelInput {
    pub name: String,
    pub color: Option<String>,
}
#[derive(Deserialize, ToSchema)]
pub struct Labels {
    pub labels: Option<Vec<LabelInput>>,
    pub names: Option<Vec<String>>,
    pub label_names: Option<Vec<String>>,
}
#[derive(Serialize, ToSchema)]
pub struct Label {
    pub id: String,
    pub issue_id: String,
    pub name: String,
    pub color: String,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct Attachment {
    pub id: String,
    pub project_issue_id: String,
    pub uploaded_by_user_id: String,
    pub file_name: String,
    pub content_type: String,
    pub byte_size: i64,
    pub storage_key: String,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateAttachment {
    pub file_name: String,
    pub content_type: Option<String>,
    pub content_base64: String,
    pub uploaded_by_user_id: Option<i64>,
}
#[derive(Serialize, ToSchema)]
pub struct AttachmentContent {
    pub attachment: Attachment,
    pub content_base64: String,
}
#[derive(ToSchema)]
pub struct MultipartAttachment {
    #[schema(value_type=String,format=Binary)]
    pub file: Vec<u8>,
}
#[derive(Serialize, ToSchema)]
pub struct UploadedAttachment {
    pub url: String,
    pub object_key: String,
    pub file_name: String,
    pub content_type: String,
    pub size: i64,
    pub markdown: String,
}
