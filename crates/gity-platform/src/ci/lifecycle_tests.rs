use super::test_support::*;
use super::*;
use axum::http::StatusCode;
#[tokio::test]
async fn runner_claim_is_atomic_expired_attempts_and_cancellation_are_enforced() {
  let (_dir, s, app, token, p) = fixture().await;
  let pipeline = post(
    &app,
    &format!("/projects/{p}/pipelines"),
    Some(&token),
    json!({}),
  )
  .await;
  let pipeline_id = pipeline["pipeline"]["id"].as_str().unwrap();
  let registration = post(
    &app,
    &format!("/projects/{p}/runners"),
    Some(&token),
    json!({"name":"runner","tags":"linux"}),
  )
  .await;
  let token = registration["token"].as_str().unwrap();
  let (a, b) = tokio::join!(
    post(&app, "/runners/jobs/claim", None, json!({"token":token})),
    post(&app, "/runners/jobs/claim", None, json!({"token":token}))
  );
  assert_ne!(a["claimed"], b["claimed"]);
  let job = if a["claimed"] == true {
    &a["job"]
  } else {
    &b["job"]
  };
  let j = job["id"].as_str().unwrap();
  s.db
    .execute(
      "UPDATE project_jobs SET locked_until=? WHERE id=?",
      vec![worker::ZERO.into(), j.parse::<i64>().unwrap().into()],
    )
    .await
    .unwrap();
  worker_tick(&s).await.unwrap();
  let claim = post(&app, "/runners/jobs/claim", None, json!({"token":token})).await;
  assert_eq!(claim["job"]["attempts"], 2);
  let heartbeat = post(&app, "/runners/heartbeat", None, json!({"token":token})).await;
  assert_eq!(heartbeat["running_job_attempts"][j], 2);
  let (status, _) = request(
    &app,
    "POST",
    &format!("/runners/jobs/{j}/complete"),
    None,
    json!({"token":token,"attempt":1}),
  )
  .await;
  assert_eq!(status, StatusCode::CONFLICT);
  let access = "test-access-token";
  post(
    &app,
    &format!("/projects/{p}/pipelines/{pipeline_id}/cancel"),
    Some(access),
    json!({}),
  )
  .await;
  let heartbeat = post(&app, "/runners/heartbeat", None, json!({"token":token})).await;
  assert_eq!(heartbeat["running_job_ids"], json!([]));
  let (status, _) = request(
    &app,
    "POST",
    &format!("/runners/jobs/{j}/complete"),
    None,
    json!({"token":token,"attempt":2}),
  )
  .await;
  assert_eq!(status, StatusCode::CONFLICT);
  let retried = post(
    &app,
    &format!("/projects/{p}/pipelines/{pipeline_id}/retry"),
    Some(access),
    json!({}),
  )
  .await;
  assert_eq!(retried["pipeline"]["status"], "pending");
  let claimed = post(&app, "/runners/jobs/claim", None, json!({"token":token})).await;
  let attempt = claimed["job"]["attempts"].as_i64().unwrap();
  post(
    &app,
    &format!("/runners/jobs/{j}/fail"),
    None,
    json!({"token":token,"attempt":attempt,"error":"test failure"}),
  )
  .await;
  let retried_job = post(
    &app,
    &format!("/projects/{p}/jobs/{j}/retry"),
    Some(access),
    json!({}),
  )
  .await;
  assert_eq!(retried_job["status"], "pending");
  let claimed = post(&app, "/runners/jobs/claim", None, json!({"token":token})).await;
  assert_eq!(claimed["job"]["id"], j);
  post(
    &app,
    &format!("/runners/jobs/{j}/complete"),
    None,
    json!({"token":token,"attempt":claimed["job"]["attempts"]}),
  )
  .await;
  assert_eq!(
    post(&app, "/runners/jobs/claim", None, json!({"token":token})).await["claimed"],
    true
  );

  let (status, _) = request(&app, "GET", &format!("/projects/{p}/jobs"), None, json!({})).await;
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn manual_noop_job_runs_and_push_pipeline_pins_yaml_commit() {
  let (_dir, s, app, token, p) = fixture().await;
  let job = post(
    &app,
    &format!("/projects/{p}/jobs"),
    Some(&token),
    json!({"kind":"noop","payload":"{}","max_attempts":1}),
  )
  .await;
  worker_tick(&s).await.unwrap();
  let j = job["id"].as_str().unwrap().parse::<i64>().unwrap();
  assert_eq!(job_row(&s, p, j).await.unwrap()["status"], "succeeded");
  let sha = s
    .git
    .resolve_revision(&format!("{p}.git"), "main", "")
    .await
    .unwrap();
  let pipeline = create_push_pipeline(&s, p, "refs/heads/main", &sha)
    .await
    .unwrap()
    .unwrap();
  assert_eq!(pipeline["pipeline"]["commit_sha"], sha);
  let duplicate = create_push_pipeline(&s, p, "main", &sha)
    .await
    .unwrap()
    .unwrap();
  assert_eq!(pipeline["pipeline"]["id"], duplicate["pipeline"]["id"]);
  let (status, _) = request(
    &app,
    "POST",
    &format!("/projects/{p}/ci/lint"),
    Some(&token),
    json!({"config_content":"pipeline { stage test {} }"}),
  )
  .await;
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn concurrent_push_admission_creates_one_complete_pipeline() {
  let (_dir, state, _app, _token, project) = fixture().await;
  let sha = state
    .git
    .resolve_revision(&format!("{project}.git"), "main", "")
    .await
    .unwrap();
  let (first, second) = tokio::join!(
    create_push_pipeline(&state, project, "refs/heads/main", &sha),
    create_push_pipeline(&state, project, "refs/heads/main", &sha)
  );
  let first = first.unwrap().unwrap();
  let second = second.unwrap().unwrap();
  assert_eq!(first["pipeline"]["id"], second["pipeline"]["id"]);
  let rows = state.ci.pipelines(project).await.unwrap();
  assert_eq!(rows.len(), 1);
  let jobs = state.ci.pipeline_jobs(project, rows[0].id).await.unwrap();
  assert_eq!(jobs.len(), 2);
  assert_eq!(
    jobs
      .iter()
      .filter(|(_, job)| job.run_after.to_string().starts_with("2037-12-31"))
      .count(),
    1
  );
}
