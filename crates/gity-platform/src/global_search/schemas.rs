use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchKind {
  Code,
  Issue,
}
impl From<SearchKind> for gity_search::global::ResourceKind {
  fn from(value: SearchKind) -> Self {
    match value {
      SearchKind::Code => Self::Code,
      SearchKind::Issue => Self::Issue,
    }
  }
}
fn kinds() -> Vec<SearchKind> {
  vec![SearchKind::Code, SearchKind::Issue]
}
fn limit() -> u16 {
  20
}
#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchRequest {
  pub query: String,
  #[serde(default = "kinds")]
  pub types: Vec<SearchKind>,
  #[serde(default)]
  pub project_ids: Vec<String>,
  pub organization_id: Option<String>,
  pub cursor: Option<String>,
  #[serde(default = "limit")]
  pub limit: u16,
}
#[derive(Serialize, ToSchema)]
pub struct SearchHit {
  pub kind: SearchKind,
  pub project_id: String,
  pub organization_id: String,
  pub project_path: String,
  pub resource_id: String,
  pub title: String,
  /// Plain text; clients must render it as text, never trusted HTML.
  pub snippet: String,
  pub path: Option<String>,
  pub commit_sha: Option<String>,
  pub line: Option<u32>,
  pub issue_iid: Option<String>,
  pub state: Option<String>,
}
#[derive(Serialize, ToSchema)]
pub struct SearchGroup {
  pub kind: SearchKind,
  pub items: Vec<SearchHit>,
}
#[derive(Serialize, ToSchema)]
pub struct SearchReply {
  pub groups: Vec<SearchGroup>,
  pub next_cursor: Option<String>,
  /// The bounded candidate or indexing budget may omit further matches.
  pub partial: bool,
  pub indexing_eventually_consistent: bool,
  pub max_code_files_per_project: usize,
  pub max_code_file_size_bytes: usize,
}
