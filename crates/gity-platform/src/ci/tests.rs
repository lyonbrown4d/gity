use super::test_support::*;
use super::*;
#[tokio::test]
async fn yaml_pipeline_runner_dag_logs_artifacts_and_completion() {
  let (_dir, s, app, token, p) = fixture().await;
  let pipeline = post(
    &app,
    &format!("/projects/{p}/pipelines"),
    Some(&token),
    json!({}),
  )
  .await;
  let pipeline_id = pipeline["pipeline"]["id"].as_str().unwrap();
  assert_eq!(pipeline["spec"]["jobs"]["test"]["needs"], json!(["build"]));
  let blocked = pipeline["jobs"]
    .as_array()
    .unwrap()
    .iter()
    .find(|job| job["pipeline_job"]["name"] == "test")
    .unwrap();
  assert_eq!(blocked["status"], "blocked");
  s.db
    .insert(
      "project_ci_variables",
      json!({"project_id":p,"key":"SECRET","value":"supersecretvalue","masked":1,"protected":1})
        .as_object()
        .unwrap()
        .clone(),
    )
    .await
    .unwrap();
  let registration = post(
    &app,
    &format!("/projects/{p}/runners"),
    Some(&token),
    json!({"name":"runner","tags":"linux"}),
  )
  .await;
  let runner = registration["token"].as_str().unwrap();
  assert!(registration["runner"].get("token").is_none());
  assert!(
    text(
      &s.db
        .one(
          "SELECT * FROM project_runners WHERE project_id=?",
          vec![p.into()]
        )
        .await
        .unwrap()
        .unwrap(),
      "token"
    )
    .starts_with("sha256:")
  );
  let claim = post(&app, "/runners/jobs/claim", None, json!({"token":runner})).await;
  assert_eq!(claim["claimed"], true);
  let j = claim["job"]["id"].as_str().unwrap();
  let attempt = claim["job"]["attempts"].as_i64().unwrap();
  let data: Value = serde_json::from_str(claim["job"]["payload"].as_str().unwrap()).unwrap();
  assert_eq!(data["env"]["SECRET"], "supersecretvalue");
  let no_more = post(&app, "/runners/jobs/claim", None, json!({"token":runner})).await;
  assert_eq!(no_more["claimed"], false);
  let archive = post(
    &app,
    &format!("/runners/jobs/{j}/source-archive"),
    None,
    json!({"token":runner,"attempt":attempt}),
  )
  .await;
  assert!(text(&archive, "content_base64").starts_with("UEs"));
  post(
    &app,
    &format!("/runners/jobs/{j}/trace"),
    None,
    json!({"token":runner,"attempt":attempt,"output":"token=supersecret"}),
  )
  .await;
  let (_, partial) = request(
    &app,
    "GET",
    &format!("/projects/{p}/jobs/{j}/trace"),
    Some(&token),
    json!({}),
  )
  .await;
  assert_eq!(partial["data"]["output"], "token=");
  post(
    &app,
    &format!("/runners/jobs/{j}/trace"),
    None,
    json!({"token":runner,"attempt":attempt,"output":"value done","final":true}),
  )
  .await;
  let (_, trace) = request(
    &app,
    "GET",
    &format!("/projects/{p}/jobs/{j}/trace"),
    Some(&token),
    json!({}),
  )
  .await;
  assert_eq!(trace["data"]["output"], "token=[MASKED] done");
  assert!(!trace.to_string().contains("supersecret"));
  let artifact=post(&app,&format!("/runners/jobs/{j}/artifacts"),None,json!({"token":runner,"attempt":attempt,"name":"result","file_name":"result.txt","content_base64":"b2s="})).await;
  let artifact_id = artifact["id"].as_str().unwrap();
  let (_, download) = request(
    &app,
    "GET",
    &format!("/projects/{p}/jobs/{j}/artifacts/{artifact_id}"),
    Some(&token),
    json!({}),
  )
  .await;
  assert_eq!(download["data"]["content_base64"], "b2s=");
  post(
    &app,
    &format!("/runners/jobs/{j}/complete"),
    None,
    json!({"token":runner,"attempt":attempt,"result":"done"}),
  )
  .await;
  let claim = post(&app, "/runners/jobs/claim", None, json!({"token":runner})).await;
  assert_eq!(
    claim["claimed"],
    true,
    "job states: {:?}",
    s.db
      .all(
        "SELECT id,status,run_after,payload FROM project_jobs WHERE project_id=?",
        vec![p.into()]
      )
      .await
      .unwrap()
  );
  let second = claim["job"]["id"].as_str().unwrap();
  post(
    &app,
    &format!("/runners/jobs/{second}/complete"),
    None,
    json!({"token":runner,"attempt":1,"result":"done"}),
  )
  .await;
  let (_, detail) = request(
    &app,
    "GET",
    &format!("/projects/{p}/pipelines/{pipeline_id}"),
    Some(&token),
    json!({}),
  )
  .await;
  assert_eq!(detail["data"]["pipeline"]["status"], "succeeded");
}
