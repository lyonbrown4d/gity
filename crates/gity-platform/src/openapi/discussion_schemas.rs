use super::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Inline review anchor: `{"file":"path","line":1,"column":1}`.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct Position {
  pub file: String,
  pub line: i64,
  pub column: Option<i64>,
}

/// Comment as it appears inside a discussion thread.
#[derive(Serialize, ToSchema)]
pub struct DiscussionComment {
  pub id: String,
  /// Thread the comment belongs to; every comment has one.
  pub discussion_id: Option<String>,
  pub author_user_id: String,
  pub body: String,
  /// Inline anchor for review comments; null for plain replies.
  pub position: Option<Position>,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}

/// One thread with its resolution state and comments, oldest comment first.
#[derive(Serialize, ToSchema)]
pub struct Discussion {
  pub id: String,
  pub resolved: bool,
  /// `null` until the thread is resolved.
  pub resolved_by_user_id: Option<String>,
  pub comments: Vec<DiscussionComment>,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}

/// Comment request body shared by issue and merge request comments.
///
/// `discussion_id` joins an existing thread; omitting it opens a new one, so a
/// plain comment and a threaded reply use the same endpoint.
#[derive(Deserialize, ToSchema)]
pub struct CreateComment {
  pub body: Option<String>,
  pub content: Option<String>,
  pub author_user_id: Option<i64>,
  pub discussion_id: Option<String>,
  pub position: Option<Position>,
}
