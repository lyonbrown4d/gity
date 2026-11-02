//! Project label request and response schemas.
use super::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A project-level label definition referenced by issues and merge requests.
#[derive(Serialize, ToSchema)]
pub struct ProjectLabel {
  pub id: String,
  pub project_id: String,
  pub title: String,
  #[schema(example = "#428BCA")]
  pub color: String,
  pub description: Option<String>,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateLabel {
  pub title: String,
  #[schema(example = "#428BCA")]
  pub color: Option<String>,
  pub description: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateLabel {
  pub title: Option<String>,
  pub color: Option<String>,
  pub description: Option<String>,
}
