use crate::{validate_path, SearchParams};
use anyhow::{bail, Result};
use regex::RegexBuilder;
use serde_json::{json, Value};

/// One matching contract shared by live scans and persistent search indexes.
pub struct SearchPlan {
  matcher: regex::Regex,
  pub path: String,
  pub limit: usize,
  pub max_files: usize,
  pub max_file_size: usize,
}
impl SearchPlan {
  pub fn new(input: &SearchParams) -> Result<Self> {
    if input.query.trim().is_empty() {
      bail!("invalid search query");
    }
    validate_path(&input.path, true)?;
    let pattern = if input.regex {
      input.query.trim().to_owned()
    } else {
      regex::escape(input.query.trim())
    };
    let matcher = RegexBuilder::new(&pattern)
      .case_insensitive(!input.match_case)
      .size_limit(8 * 1024 * 1024)
      .build()
      .map_err(|e| anyhow::anyhow!("invalid search regex: {e}"))?;
    Ok(Self {
      matcher,
      path: input.path.clone(),
      limit: bounded(input.limit, 20, 500),
      max_files: bounded(input.max_files, 300, 5000),
      max_file_size: bounded(input.max_file_size, 256 * 1024, 10 * 1024 * 1024),
    })
  }
  pub fn in_scope(&self, path: &str) -> bool {
    self.path.is_empty()
      || path == self.path
      || path
        .strip_prefix(&self.path)
        .is_some_and(|suffix| suffix.starts_with('/'))
  }
  pub fn append_matches(&self, path: &str, content: &[u8], results: &mut Vec<Value>) {
    if results.len() >= self.limit || content.len() > self.max_file_size || !self.in_scope(path) {
      return;
    }
    let Ok(text) = std::str::from_utf8(content) else {
      return;
    };
    if text.contains('\0') {
      return;
    }
    for (number, line) in text.split('\n').enumerate() {
      if let Some(found) = self.matcher.find(line) {
        results.push(json!({"path":path,"line_number":number+1,"column":line[..found.start()].chars().count()+1,"match_length":found.as_str().chars().count(),"line_content":line}));
        if results.len() >= self.limit {
          return;
        }
      }
    }
  }
}
fn bounded(value: usize, default: usize, max: usize) -> usize {
  if value == 0 {
    default
  } else {
    value.min(max)
  }
}
