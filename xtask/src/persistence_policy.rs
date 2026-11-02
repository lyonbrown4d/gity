//! Keep HTTP adapters and domain commands independent of persistence mechanics.
use std::path::Path;

pub fn check(path: &Path, source: &str, is_test: bool, violations: &mut Vec<String>) {
  if is_test
    || path
      .file_name()
      .is_some_and(|name| name == "test_support.rs")
  {
    return;
  }
  let path_text = path.to_string_lossy().replace('\\', "/");
  if path_text.contains("/gity-platform/src/") || path_text.contains("/apps/gity-search-index/src/")
  {
    let compact = source.split_whitespace().collect::<String>();
    for method in [
      "one",
      "all",
      "insert",
      "update",
      "delete",
      "execute",
      "statement",
      "next_iid",
    ] {
      if compact.contains(&format!(".db.{method}(")) {
        violations.push(format!(
          "{}: generic Db::{method} is restricted to test fixtures",
          path.display()
        ));
      }
    }
    for forbidden in [
      "SELECT ",
      "INSERT INTO ",
      "DELETE FROM ",
      "UPDATE ",
      "Entity::",
      "ActiveModel",
    ] {
      if source.contains(forbidden) {
        violations.push(format!(
          "{}: HTTP/platform adapters must use typed application services ({forbidden})",
          path.display()
        ));
      }
    }
  }
  let context = ["gity-ci", "gity-collaboration", "gity-projects"]
    .iter()
    .any(|name| path_text.contains(&format!("/{name}/src/")));
  let domain = path_text.contains("/domain/") || path_text.ends_with("/domain.rs");
  if context && domain {
    for dependency in ["sea_orm", "axum", "tonic"] {
      if source.contains(dependency) {
        violations.push(format!(
          "{}: domain commands cannot depend on {dependency}",
          path.display()
        ));
      }
    }
  }
}
