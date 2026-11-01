use super::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, ToSchema)]
pub struct Tag {
    pub name: String,
    pub target_sha: String,
    pub object_sha: String,
    pub object_type: String,
    pub annotated: bool,
    pub message: String,
    pub created_at: String,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateTag {
    pub name: String,
    pub source_ref: Option<String>,
}
#[derive(Serialize, ToSchema)]
pub struct Release {
    pub id: String,
    pub project_id: String,
    pub tag_name: String,
    pub name: String,
    pub description: String,
    pub created_by_user_id: String,
    pub released_at: String,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct ReleaseLink {
    pub id: String,
    pub project_release_id: String,
    pub name: String,
    pub url: String,
    pub link_type: String,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct ReleaseDetail {
    pub release: Release,
    pub links: Vec<ReleaseLink>,
    pub tag: Option<Tag>,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub source_ref: Option<String>,
    pub create_tag: Option<bool>,
    pub released_at: Option<String>,
    pub created_by_user_id: Option<i64>,
}
#[derive(Deserialize, ToSchema)]
pub struct UpdateRelease {
    pub name: Option<String>,
    pub description: Option<String>,
    pub released_at: Option<String>,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateReleaseLink {
    pub name: String,
    pub url: String,
    pub link_type: Option<String>,
}
