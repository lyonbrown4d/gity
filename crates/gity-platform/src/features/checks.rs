use super::codeowners::owners;
use super::merge_requests::{branch, git_error, repository};
use super::*;
pub(super) fn matches(pattern: &str, value: &str) -> bool {
  if pattern.is_empty() || pattern == "*" {
    return true;
  }
  let mut rx = String::from("^");
  let mut chars = pattern.chars().peekable();
  while let Some(c) = chars.next() {
    match c {
      '*' => {
        if chars.peek() == Some(&'*') {
          chars.next();
          if chars.peek() == Some(&'/') {
            chars.next();
            rx.push_str("(?:.*/)?");
          } else {
            rx.push_str(".*");
          }
        } else {
          rx.push_str("[^/]*");
        }
      }
      '[' => {
        let mut class = String::new();
        if chars.peek().is_some_and(|c| *c == '!' || *c == '^') {
          chars.next();
          class.push('^');
        }
        let mut closed = false;
        for c in chars.by_ref() {
          if c == ']' {
            closed = true;
            break;
          }
          if c == '\\' || c == '[' {
            class.push('\\');
          }
          class.push(c);
        }
        if !closed {
          return pattern == value;
        }
        rx.push('[');
        rx.push_str(&class);
        rx.push(']');
      }
      '?' => rx.push_str("[^/]"),
      _ => rx.push_str(&regex::escape(&c.to_string())),
    }
  }
  rx.push('$');
  regex::Regex::new(&rx)
    .map(|re| re.is_match(value))
    .unwrap_or(pattern == value)
}
fn blocker(code: &str, category: &str, message: &str) -> Value {
  json!({"code":code,"category":category,"message":message})
}
fn rule_check(rule: &Value, eligible: &[i64], approvers: &[i64]) -> Value {
  let count = approvers
    .iter()
    .filter(|v| eligible.is_empty() || eligible.contains(v))
    .count();
  let needed = number(rule, "approvals_required").max(1);
  let satisfied = count as i64 >= needed;
  json!({
      "rule_id":number(rule, "id").to_string(),
      "name":rule["name"],
      "target_branch":rule["target_branch"],
      "approvals_required":needed,
      "approval_count":count,
      "eligible_user_ids":eligible.iter().map(|v|v.to_string()).collect::<Vec<_>>(),
      "code_owner":number(rule, "code_owner")!=0,
      "satisfied":satisfied,
      "blocking_reason":if satisfied{String::new()}else{format!("approval rule {} requires approval",string(rule,"name"))},
  })
}
pub(super) async fn evaluate(s: &AppState, p: &Value, mr: &Value) -> Result<Value, AppError> {
  let project = number(p, "id");
  let source = string(mr, "source_branch");
  let target = string(mr, "target_branch");
  let sha = branch(s, p, source).await?;
  branch(s, p, target).await?;
  let protections = s
    .db
    .all(
      "SELECT * FROM project_branch_protections WHERE project_id = ? ORDER BY id",
      vec![project.into()],
    )
    .await?;
  let protection = crate::repository::effective_rule(&protections, target);
  let require_mr = protection
    .map(|v| number(v, "require_merge_request") != 0)
    .unwrap_or(false);
  let has_ci = s
    .git
    .file_exists(&repository(p), &sha, "", ".gity-ci.yml")
    .await
    .map_err(git_error)?;
  let pipeline_required = protection
    .map(|v| number(v, "require_pipeline_success") != 0)
    .unwrap_or(false)
    || has_ci;
  let mut blockers = Vec::new();
  let mut status = "not_required".to_owned();
  let mut pipeline = Value::Null;
  if pipeline_required {
    if let Some(row) = s.db.one(
            "SELECT * FROM project_pipelines WHERE project_id = ? AND ref_name = ? AND commit_sha = ? ORDER BY id DESC LIMIT 1",
            vec![project.into(), source.into(), sha.clone().into()],
        ).await? {
            status = string(&row, "status").to_owned();
            if has_ci {
                let pinned = s.git.get_blob_limited(&repository(p), &sha, "", ".gity-ci.yml", 1024 * 1024)
                    .await.map_err(git_error)?;
                if pinned["encoding"] != "utf-8" || string(&pinned, "content") != string(&row, "config_content") {
                    status = "untrusted".into();
                    blockers.push(blocker("pipeline_config_mismatch", "pipeline", "pipeline configuration differs from the source commit"));
                }
            }
            if status != "succeeded" && status != "untrusted" {
                blockers.push(blocker("pipeline_not_succeeded", "pipeline", &format!("pipeline status is {status}")));
            }
            pipeline = row;
        } else {
            status = "missing".into();
            blockers.push(blocker("pipeline_missing", "pipeline", "required pipeline is missing"));
        }
  }
  let approvals=s.db.all("SELECT user_id FROM project_merge_request_approvals WHERE merge_request_id = ? AND user_id <> ?",vec![number(mr,"id").into(),number(mr,"author_user_id").into()]).await?;
  let approvers = approvals
    .iter()
    .map(|v| number(v, "user_id"))
    .collect::<Vec<_>>();
  let mut checks = Vec::new();
  if require_mr {
    checks.push(rule_check(
      &json!({"name":"Protected branch approval","target_branch":target,"approvals_required":1}),
      &[],
      &approvers,
    ));
  }
  for rule in s
    .db
    .all(
      "SELECT * FROM project_merge_request_approval_rules WHERE project_id = ? ORDER BY id",
      vec![project.into()],
    )
    .await?
  {
    if !matches(string(&rule, "target_branch"), target) {
      continue;
    }
    let eligible = if number(&rule, "code_owner") != 0 {
      owners(s, p, mr).await?
    } else {
      super::rules::ids(&rule)
    };
    if number(&rule, "code_owner") != 0 && eligible.is_empty() {
      continue;
    }
    checks.push(rule_check(&rule, &eligible, &approvers));
  }
  for check in &checks {
    if check["satisfied"] != true {
      blockers.push(blocker(
        "approval_rule_unsatisfied",
        "approval",
        string(check, "blocking_reason"),
      ));
    }
  }
  let mergeable = blockers.is_empty();
  if !checks.is_empty() && status == "not_required" {
    status = if mergeable { "passed" } else { "blocked" }.into();
  }
  let required_approvals = checks
    .iter()
    .map(|v| number(v, "approvals_required"))
    .sum::<i64>();
  Ok(json!({
      "merge_request":mr,
      "source_branch":source,
      "source_commit_sha":sha,
      "target_branch":target,
      "target_branch_protected":protection.is_some(),
      "require_merge_request":require_mr,
      "require_pipeline_success":protection.map(|v|number(v, "require_pipeline_success")!=0).unwrap_or(false),
      "require_approval":!checks.is_empty(),
      "required_approvals":required_approvals,
      "approval_count":approvers.len(),
      "approval_rules":checks,
      "merge_access_level":protection.map(|v|string(v, "merge_access_level")).unwrap_or(""),
      "pipeline_required":pipeline_required,
      "required":pipeline_required||required_approvals>0,
      "mergeable":mergeable,
      "status":status,
      "blocking_reason":blockers.first().map(|v|string(v, "message")).unwrap_or(""),
      "blockers":blockers,
      "pipeline":pipeline,
  }))
}
#[cfg(test)]
#[path = "checks_tests.rs"]
mod tests;
