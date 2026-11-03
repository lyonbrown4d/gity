//! Engine-neutral documents and queries. Engines never decide resource access.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
  Code,
  Issue,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchDocument {
  pub key: String,
  pub project_id: i64,
  pub kind: ResourceKind,
  /// Issue primary key, or SHA-256 content digest for code.
  pub resource_id: String,
  /// Issue updated_at string, or default-branch commit SHA for code.
  pub version: String,
  pub title: String,
  pub body: String,
  pub path: String,
  pub iid: Option<i64>,
  pub line_start: usize,
  pub confidential: bool,
}
#[derive(Clone, Debug)]
pub struct EngineQuery {
  pub query: String,
  pub project_ids: Vec<i64>,
  pub namespaces: Vec<(i64, String)>,
  pub confidential_projects: Vec<i64>,
  pub confidential_issue_ids: Vec<i64>,
  pub kind: ResourceKind,
  /// Zero-based candidate offset, internal only; bounded by application limits.
  pub offset: usize,
  pub limit: usize,
}
#[derive(Clone, Debug, Default)]
pub struct CandidatePage {
  pub documents: Vec<SearchDocument>,
  pub has_more: bool,
  pub partial: bool,
}

/// Case-insensitive Unicode substring matching shared by both engines.
pub fn matches(query: &str, text: &str) -> bool {
  text.to_lowercase().contains(&query.to_lowercase())
}
