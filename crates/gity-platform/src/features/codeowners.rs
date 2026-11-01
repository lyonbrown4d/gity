use super::checks::matches;
use super::merge_requests::{git_error, repository};
use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
pub(super) async fn owners(s: &AppState, p: &Value, mr: &Value) -> Result<Vec<i64>, AppError> {
  let mut content = String::new();
  for path in ["CODEOWNERS", ".gitlab/CODEOWNERS", "docs/CODEOWNERS"] {
    if s
      .git
      .file_exists(
        &repository(p),
        string(mr, "target_branch"),
        string(p, "default_branch"),
        path,
      )
      .await
      .map_err(git_error)?
    {
      let blob = s
        .git
        .get_blob(
          &repository(p),
          string(mr, "target_branch"),
          string(p, "default_branch"),
          path,
        )
        .await
        .map_err(git_error)?;
      content = if string(&blob, "encoding") == "base64" {
        String::from_utf8(
          STANDARD
            .decode(string(&blob, "content"))
            .map_err(|_| AppError::internal("invalid CODEOWNERS content"))?,
        )
        .map_err(|_| AppError::internal("invalid CODEOWNERS encoding"))?
      } else {
        string(&blob, "content").into()
      };
      break;
    }
  }
  if content.is_empty() {
    return Ok(Vec::new());
  }
  let rules = content.lines().filter_map(parse_owner).collect::<Vec<_>>();
  let diff = s
    .git
    .diff_branches(
      &repository(p),
      string(mr, "target_branch"),
      string(mr, "source_branch"),
    )
    .await
    .map_err(git_error)?;
  let mut usernames = std::collections::HashSet::new();
  for file in diff.lines().filter_map(|line| line.strip_prefix("+++ b/")) {
    for (pattern, owners) in rules.iter().rev() {
      let pattern = pattern.trim_start_matches('/');
      let matched = matches(pattern, file)
        || (!pattern.contains('/') && matches(&format!("**/{pattern}*"), file))
        || (pattern.ends_with('/') && matches(&format!("{pattern}**"), file));
      if matched {
        usernames.extend(owners.iter().cloned());
        break;
      }
    }
  }
  let mut ids = Vec::new();
  for user in usernames {
    if let Some(row) = s
      .db
      .one("SELECT id FROM users WHERE username = ?", vec![user.into()])
      .await?
    {
      ids.push(number(&row, "id"));
    }
  }
  Ok(ids)
}
pub(super) fn parse_owner(line: &str) -> Option<(String, Vec<String>)> {
  let mut fields = Vec::new();
  let mut field = String::new();
  let mut escaped = false;
  let mut start = true;
  for c in line.trim().chars() {
    if escaped {
      field.push(c);
      escaped = false;
      start = false;
      continue;
    }
    if c == '\\' {
      escaped = true;
      continue;
    }
    if c == '#' && start {
      break;
    }
    if c.is_whitespace() {
      if !field.is_empty() {
        fields.push(std::mem::take(&mut field));
      }
      start = true;
    } else {
      field.push(c);
      start = false;
    }
  }
  if !field.is_empty() {
    fields.push(field);
  }
  if fields.len() < 2 || fields[0].starts_with(['[', '!']) {
    return None;
  }
  let names = fields[1..]
    .iter()
    .filter_map(|v| v.strip_prefix('@'))
    .filter_map(|v| {
      let name = v
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || ['_', '.', '-'].contains(c))
        .collect::<String>();
      (!name.is_empty()).then(|| name.to_lowercase())
    })
    .collect::<Vec<_>>();
  if names.is_empty() {
    None
  } else {
    Some((fields[0].clone(), names))
  }
}
