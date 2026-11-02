//! Milestone request and response schemas.
use super::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Milestone {
  /// Application-generated milestone identifier serialized as a string.
  pub id: String,
  pub project_id: String,
  /// Project-local milestone number used in the request path.
  pub iid: i64,
  pub title: String,
  pub description: Option<String>,
  #[schema(example = "active")]
  pub state: String,
  /// Calendar day in `YYYY-MM-DD` form.
  pub start_date: Option<String>,
  /// Calendar day in `YYYY-MM-DD` form.
  pub due_date: Option<String>,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateMilestone {
  pub title: String,
  pub description: Option<String>,
  #[schema(example = "active")]
  pub state: Option<String>,
  pub start_date: Option<String>,
  pub due_date: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateMilestone {
  pub title: Option<String>,
  pub description: Option<String>,
  #[schema(example = "closed")]
  pub state: Option<String>,
  pub start_date: Option<String>,
  pub due_date: Option<String>,
}
