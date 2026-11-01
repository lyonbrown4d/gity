use super::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, ToSchema)]
pub struct WikiPage {
  pub id: String,
  pub project_id: String,
  pub slug: String,
  pub title: String,
  pub content: String,
  pub format: String,
  pub author_user_id: String,
  pub last_edited_by_user_id: String,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Deserialize, ToSchema)]
pub struct CreateWikiPage {
  pub slug: Option<String>,
  pub title: String,
  pub content: Option<String>,
  #[schema(example = "markdown")]
  pub format: Option<String>,
  pub author_user_id: Option<i64>,
}
#[derive(Deserialize, ToSchema)]
pub struct UpdateWikiPage {
  pub title: Option<String>,
  pub content: Option<String>,
  pub editor_user_id: Option<i64>,
}
#[derive(Serialize, ToSchema)]
pub struct Todo {
  pub id: String,
  pub user_id: String,
  pub organization_id: String,
  pub project_id: String,
  pub kind: String,
  pub state: String,
  pub target_type: String,
  pub target_id: String,
  pub title: String,
  pub summary: String,
  pub action_url: String,
  #[serde(flatten)]
  pub timestamps: Timestamps,
  pub done_at: Option<String>,
}
#[derive(Serialize, ToSchema)]
pub struct TodoDone {
  pub id: String,
  pub state: String,
}
