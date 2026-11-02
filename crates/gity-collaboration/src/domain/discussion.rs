use serde::Deserialize;

/// Discussion owner as stored in `project_discussions.target_type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscussionTarget {
  Issue,
  MergeRequest,
}

impl DiscussionTarget {
  pub fn parse(raw: &str) -> Option<Self> {
    match raw {
      "issue" => Some(Self::Issue),
      "merge_request" => Some(Self::MergeRequest),
      _ => None,
    }
  }

  pub fn as_str(self) -> &'static str {
    match self {
      Self::Issue => "issue",
      Self::MergeRequest => "merge_request",
    }
  }
}

/// A thread opened for one issue or merge request.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct DiscussionCreate {
  pub project_id: i64,
  pub target_type: String,
  pub target_id: i64,
}

/// Resolution change requested by an authorized actor.
///
/// `can_write` carries the platform decision that the actor holds the issue or
/// merge request write action; discussion authors may always resolve their own
/// thread, so the permission itself stays outside this crate.
#[derive(Clone, Copy, Debug)]
pub struct DiscussionResolve {
  pub project_id: i64,
  pub target: DiscussionTarget,
  pub iid: i64,
  pub discussion_id: i64,
  pub actor_user_id: i64,
  pub can_write: bool,
  pub resolved: bool,
}
