use crate::ci::{test_support::*, *};
use axum::http::StatusCode;

#[tokio::test]
async fn bounded_trace_selects_attempt_and_slices_utf8_without_hydrating_other_outputs() {
  let (_dir, state, app, owner, project) = fixture().await;
  let job = post(
    &app,
    &format!("/projects/{project}/jobs"),
    Some(&owner),
    json!({"kind":"noop","payload":"{}"}),
  )
  .await;
  let job_id = text(&job, "id");
  let output = format!("A🦀中Z{}", "x".repeat(300 * 1024));
  for (attempt, body) in [(1, output.as_str()), (2, "latest")] {
    state
      .db
      .insert(
        "project_job_logs",
        json!({
          "project_id":project,"project_job_id":job_id.parse::<i64>().unwrap(),
          "attempt":attempt,"exit_code":0,"output":body,"output_truncated":i64::from(attempt == 1),
          "duration_millis":123
        })
        .as_object()
        .unwrap()
        .clone(),
      )
      .await
      .unwrap();
  }
  let url = format!("/projects/{project}/jobs/{job_id}/trace");
  let (status, page) = request(
    &app,
    "GET",
    &format!("{url}?attempt=1&offset=0&limit=4"),
    Some(&owner),
    json!({}),
  )
  .await;
  assert_eq!(status, StatusCode::OK, "{page}");
  let data = &page["data"];
  assert_eq!(data["attempt"], 1);
  assert_eq!(data["output"], "A", "{page}");
  assert_eq!(data["log"]["output"], "A");
  assert_eq!(data["next_offset"], 1);
  assert_eq!(data["total_bytes"], output.len());
  assert_eq!(data["has_more"], true);
  assert_eq!(data["truncated"], true);
  assert_eq!(data["attempts"], json!([]));
  assert_eq!(data["attempt_summaries"].as_array().unwrap().len(), 2);
  for summary in data["attempt_summaries"].as_array().unwrap() {
    assert_eq!(summary["output"], "");
    assert_eq!(summary["duration_millis"], 123);
  }
  let (_, next) = request(
    &app,
    "GET",
    &format!("{url}?attempt=1&offset=1&limit=4"),
    Some(&owner),
    json!({}),
  )
  .await;
  assert_eq!(next["data"]["output"], "🦀");
  assert_eq!(next["data"]["next_offset"], 5);
  let (_, split) = request(
    &app,
    "GET",
    &format!("{url}?attempt=1&offset=2&limit=4"),
    Some(&owner),
    json!({}),
  )
  .await;
  assert_eq!(split["data"]["offset"], 5);
  assert_eq!(split["data"]["output"], "中Z");
  let (_, latest) = request(
    &app,
    "GET",
    &format!("{url}?offset=0"),
    Some(&owner),
    json!({}),
  )
  .await;
  assert_eq!(latest["data"]["attempt"], 2);
  assert_eq!(latest["data"]["output"], "latest");
  assert_eq!(latest["data"]["has_more"], false);
  let (_, large) = request(
    &app,
    "GET",
    &format!("{url}?attempt=1"),
    Some(&owner),
    json!({}),
  )
  .await;
  assert!(text(&large["data"], "output").len() <= 64 * 1024);
  let (_, eof) = request(
    &app,
    "GET",
    &format!("{url}?attempt=1&offset=999999"),
    Some(&owner),
    json!({}),
  )
  .await;
  assert_eq!(eof["data"]["output"], "");
  assert_eq!(eof["data"]["next_offset"], output.len());
  assert_eq!(eof["data"]["has_more"], false);
  let (_, missing) = request(
    &app,
    "GET",
    &format!("{url}?attempt=3"),
    Some(&owner),
    json!({}),
  )
  .await;
  assert!(missing["data"]["log"].is_null());
  assert_eq!(missing["data"]["attempt"], 3);
  for query in [
    "attempt=0",
    "limit=0",
    "limit=262145",
    "offset=18446744073709551615",
  ] {
    assert_eq!(
      request(
        &app,
        "GET",
        &format!("{url}?{query}"),
        Some(&owner),
        json!({})
      )
      .await
      .0,
      StatusCode::BAD_REQUEST,
      "{query}"
    );
  }
  assert_eq!(
    request(&app, "GET", &format!("{url}?limit=4"), None, json!({}))
      .await
      .0,
    StatusCode::UNAUTHORIZED
  );
  let other = format!("/projects/{}/jobs/{job_id}/trace?limit=4", project + 1);
  assert_eq!(
    request(&app, "GET", &other, Some(&owner), json!({}))
      .await
      .0,
    StatusCode::NOT_FOUND
  );
}

#[tokio::test]
async fn editor_file_commit_reports_stale_head_as_conflict() {
  let (_dir, state, _app, owner, project) = fixture().await;
  let path = format!("/projects/{project}/repository/file-commits");
  let head = state
    .git
    .resolve_revision(&format!("{project}.git"), "main", "")
    .await
    .unwrap();
  let app = crate::repository::routes().with_state(state);
  let body = json!({"branch_name":"main","file_path":".gity-ci.yml","content":"jobs: {}\n",
    "message":"Edit CI","expected_head_sha":head,"update_existing":true});
  let (status, saved) = request(&app, "POST", &path, Some(&owner), body.clone()).await;
  assert_eq!(status, StatusCode::OK, "{saved}");
  assert_ne!(saved["data"]["commit_sha"], head);
  let (status, stale) = request(&app, "POST", &path, Some(&owner), body).await;
  assert_eq!(status, StatusCode::CONFLICT, "{stale}");
  assert_eq!(
    request(
      &app,
      "POST",
      &path,
      Some(&owner),
      json!({
        "file_path":".gity-ci.yml","content":"overwrite","message":"Unsafe overwrite"
      })
    )
    .await
    .0,
    StatusCode::CONFLICT
  );
}
