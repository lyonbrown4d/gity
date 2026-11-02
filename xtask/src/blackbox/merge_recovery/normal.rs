use super::*;

pub(super) fn run(f: &Fixture<'_>, reporter: &mut Reporter) -> Result<()> {
  let reviewer_id = f.reviewer_id.to_string();
  let issue = f.ok(Method::POST, "/issues", f.owner, Some(json!({
    "title":"Verify durable collaboration","description":"Exercise comments, labels and milestone","milestone_id":f.milestone
  })))?;
  let iid = api::id(&issue["number"], "issue IID")?;
  let path = format!("/issues/{iid}");
  f.ok(
    Method::PATCH,
    &format!("{path}/labels"),
    f.owner,
    Some(json!({"names":["recovery"]})),
  )?;
  f.ok(
    Method::POST,
    &format!("{path}/comments"),
    &f.reviewer,
    Some(json!({"body":"Reviewed issue details"})),
  )?;
  f.ok(
    Method::PATCH,
    &format!("{path}/assignees"),
    f.owner,
    Some(json!({"user_ids":[f.reviewer_id.to_string()]})),
  )?;
  let assigned = f.get(&format!("{path}/assignees"))?;
  ensure!(
    assigned.as_array().is_some_and(|rows| rows
      .iter()
      .any(|row| row["user_id"].as_str() == Some(reviewer_id.as_str()))),
    "issue assignee was not retained"
  );
  let detail = f.get(&path)?;
  ensure!(
    detail["milestone_id"] == f.milestone,
    "issue milestone association was not retained"
  );
  let labels = f.get(&format!("{path}/labels"))?;
  ensure!(
    labels
      .as_array()
      .is_some_and(|rows| rows.iter().any(|row| row["name"] == "recovery")),
    "issue label association missing"
  );
  let comments = f.get(&format!("{path}/comments"))?;
  ensure!(
    comments.as_array().is_some_and(|rows| rows
      .iter()
      .any(|row| row["content"] == "Reviewed issue details")),
    "issue comment missing"
  );
  let updated = f.ok(
    Method::PATCH,
    &path,
    f.owner,
    Some(json!({"status":"closed"})),
  )?;
  ensure!(updated["status"] == "closed", "issue close failed");
  let updated = f.ok(
    Method::PATCH,
    &path,
    f.owner,
    Some(json!({"status":"open"})),
  )?;
  ensure!(updated["status"] == "open", "issue reopen failed");
  for (method, suffix, body) in [
    (Method::GET, path.clone(), None),
    (
      Method::POST,
      format!("{path}/comments"),
      Some(json!({"body":"outsider"})),
    ),
    (
      Method::PATCH,
      format!("{path}/labels"),
      Some(json!({"names":["intruder"]})),
    ),
    (
      Method::POST,
      "/milestones".into(),
      Some(json!({"title":"intruder"})),
    ),
    (
      Method::POST,
      "/labels".into(),
      Some(json!({"title":"intruder"})),
    ),
  ] {
    api::require_denied(
      f.request(method, &suffix, f.outsider, body)?.0,
      "outsider collaboration mutation/read",
    )?;
  }
  reporter.pass(
    "collaboration-issue-label-milestone",
    "issue comment, assignee, project label, milestone, close/reopen and outsider denials passed",
  );

  let mr = f.create_merge("reviewed-feature")?;
  let iid = api::id(&mr["iid"], "merge IID")?;
  let path = format!("/merge-requests/{iid}");
  f.ok(
    Method::PATCH,
    &format!("{path}/reviewers"),
    f.owner,
    Some(json!({"user_ids":[f.reviewer_id.to_string()]})),
  )?;
  let participants = f.get(&format!("{path}/participants"))?;
  ensure!(
    participants["participants"]
      .as_array()
      .is_some_and(|rows| rows.iter().any(
        |row| row["role"] == "reviewer" && row["user_id"].as_str() == Some(reviewer_id.as_str())
      )),
    "MR reviewer missing"
  );
  f.ok(
    Method::POST,
    &format!("{path}/comments"),
    &f.reviewer,
    Some(json!({"body":"Review complete"})),
  )?;
  let comments = f.get(&format!("{path}/comments"))?;
  ensure!(
    comments["comments"]
      .as_array()
      .is_some_and(|rows| rows.iter().any(|row| row["body"] == "Review complete")),
    "MR review comment missing"
  );
  f.ok(
    Method::POST,
    &format!("{path}/approve"),
    f.owner,
    Some(json!({})),
  )?;
  let checks = f.get(&format!("{path}/checks"))?;
  ensure!(
    checks["mergeable"] == false,
    "author approval unexpectedly satisfied independent reviewer rule"
  );
  api::require_status(
    f.request(
      Method::POST,
      &format!("{path}/merge"),
      f.owner,
      Some(json!({})),
    )?
    .0,
    StatusCode::CONFLICT,
    "unapproved merge",
  )?;
  for suffix in ["approve", "merge", "merge/abort"] {
    api::require_denied(
      f.request(
        Method::POST,
        &format!("{path}/{suffix}"),
        f.outsider,
        Some(json!({"reason":"outsider"})),
      )?
      .0,
      "outsider MR action",
    )?;
  }
  f.approve(&iid)?;
  let approvals = f.get(&format!("{path}/approvals"))?;
  ensure!(
    approvals["approvals"].as_array().is_some_and(|rows| rows
      .iter()
      .any(|row| row["user_id"].as_str() == Some(reviewer_id.as_str()))),
    "reviewer approval missing"
  );
  ensure!(
    f.get(&format!("{path}/checks"))?["mergeable"] == true,
    "approved MR still blocked"
  );
  let merged = f.ok(
    Method::POST,
    &format!("{path}/merge"),
    &f.reviewer,
    Some(json!({})),
  )?;
  ensure!(merged["state"] == "merged", "normal merge did not complete");
  ensure!(
    merged["merge_commit_sha"] == f.main_sha()?,
    "normal merge did not return the Git commit"
  );
  ensure!(
    merged["milestone_id"] == f.milestone,
    "MR milestone association lost"
  );
  reporter.pass("collaboration-mr-review-and-merge", "reviewer assignment, comment, approval gate, successful merge and outsider action isolation passed");
  Ok(())
}
