use super::{test_support::*, *};
use axum::{
  body::{Body, to_bytes},
  http::{HeaderMap, Request, StatusCode},
};
use gity_entities::project_jobs;
use sea_orm::EntityTrait;
use tower::ServiceExt;

async fn agent_request(app: &Router, path: &str, job_token: &str, body: Value) -> Value {
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .header("JOB-TOKEN", job_token)
        .body(Body::from(body.to_string()))
        .unwrap(),
    )
    .await
    .unwrap();
  let status = response.status();
  let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
    .await
    .unwrap();
  let value: Value = serde_json::from_slice(&bytes).unwrap();
  assert_eq!(status, StatusCode::OK, "{value}");
  value["data"].clone()
}

#[tokio::test]
async fn job_credentials_are_transient_project_scoped_attempt_scoped_and_redacted_across_chunks() {
  let (_dir, state, app, owner, project) = fixture().await;
  let created = post(
    &app,
    &format!("/projects/{project}/jobs"),
    Some(&owner),
    json!({"kind":"script", "payload":json!({"script":["true"]}).to_string()}),
  )
  .await;
  let job_id = created["id"].as_str().unwrap();
  let job_id_number: i64 = job_id.parse().unwrap();
  let registration = post(
    &app,
    &format!("/projects/{project}/runners"),
    Some(&owner),
    json!({"name":"package-agent"}),
  )
  .await;
  let runner = registration["token"].as_str().unwrap();
  let claimed = post(
    &app,
    "/runners/jobs/claim",
    None,
    json!({"token":runner,"job_package_token":true}),
  )
  .await;
  assert_eq!(claimed["job"]["id"], job_id);
  let payload: Value = serde_json::from_str(claimed["job"]["payload"].as_str().unwrap()).unwrap();
  let raw = payload["env"]["CI_JOB_TOKEN"].as_str().unwrap();
  assert!(raw.starts_with("gity_job_v1."));
  assert!(payload[package_identity::HASH_FIELD].is_null());
  let stored = project_jobs::Entity::find_by_id(job_id_number)
    .one(&state.db.connection)
    .await
    .unwrap()
    .unwrap();
  assert!(!stored.payload.unwrap().contains(raw));
  let mut headers = HeaderMap::new();
  headers.insert("JOB-TOKEN", raw.parse().unwrap());
  let identity = package_identity(&state, &headers, project, true)
    .await
    .unwrap()
    .unwrap();
  assert_eq!(identity.job_id, job_id_number);
  assert!(
    package_identity(&state, &headers, project + 1, false)
      .await
      .is_err()
  );
  let (status, public) = request(
    &app,
    "GET",
    &format!("/projects/{project}/jobs/{job_id}"),
    Some(&owner),
    Value::Null,
  )
  .await;
  assert_eq!(status, StatusCode::OK);
  assert!(!public.to_string().contains(raw));
  assert!(!public.to_string().contains(package_identity::HASH_FIELD));
  let midpoint = raw.len() / 2;
  agent_request(
    &app,
    &format!("/runners/jobs/{job_id}/trace"),
    raw,
    json!({"token":runner,"attempt":1,"output":format!("before {}",&raw[..midpoint])}),
  )
  .await;
  agent_request(
    &app,
    &format!("/runners/jobs/{job_id}/trace"),
    raw,
    json!({"token":runner,"attempt":1,"output":format!("{} after",&raw[midpoint..]),"final":true}),
  )
  .await;
  let logs = state
    .db
    .one(
      "SELECT output FROM project_job_logs WHERE project_job_id=?",
      vec![job_id_number.into()],
    )
    .await
    .unwrap()
    .unwrap();
  assert_eq!(logs["output"], "before [MASKED] after");
  agent_request(
    &app,
    &format!("/runners/jobs/{job_id}/fail"),
    raw,
    json!({"token":runner,"attempt":1,"result":raw,"error":raw,"exit_code":1}),
  )
  .await;
  assert!(
    package_identity(&state, &headers, project, false)
      .await
      .is_err()
  );
  let finished = project_jobs::Entity::find_by_id(job_id_number)
    .one(&state.db.connection)
    .await
    .unwrap()
    .unwrap();
  assert_eq!(finished.result.as_deref(), Some("[MASKED]"));
  post(
    &app,
    &format!("/projects/{project}/jobs/{job_id}/retry"),
    Some(&owner),
    json!({}),
  )
  .await;
  let retry = post(
    &app,
    "/runners/jobs/claim",
    None,
    json!({"token":runner,"job_package_token":true}),
  )
  .await;
  assert_eq!(retry["job"]["attempts"], 2);
  let payload: Value = serde_json::from_str(retry["job"]["payload"].as_str().unwrap()).unwrap();
  let next = payload["env"]["CI_JOB_TOKEN"].as_str().unwrap();
  assert_ne!(next, raw);
  assert!(
    package_identity(&state, &headers, project, true)
      .await
      .is_err()
  );
  headers.insert("JOB-TOKEN", next.parse().unwrap());
  assert!(
    package_identity(&state, &headers, project, true)
      .await
      .unwrap()
      .is_some()
  );
}
