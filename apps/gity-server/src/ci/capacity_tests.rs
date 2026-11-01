use super::test_support::*;
use super::*;

#[tokio::test]
async fn yaml_scripts_and_trace_chunks_round_trip_beyond_mysql_text_capacity() {
    let (_dir, _state, app, owner, project) = fixture().await;
    let script = format!("#{}\necho ok", "s".repeat(80 * 1024));
    let yaml = json!({"jobs":{"large":{"script":script}}}).to_string();
    let pipeline = post(
        &app,
        &format!("/projects/{project}/pipelines"),
        Some(&owner),
        json!({"config_content":yaml}),
    )
    .await;
    assert_eq!(pipeline["pipeline"]["config_content"], yaml);
    assert_eq!(pipeline["jobs"][0]["script"][0], script);
    let registration = post(
        &app,
        &format!("/projects/{project}/runners"),
        Some(&owner),
        json!({"name":"large-content-runner"}),
    )
    .await;
    let token = text(&registration, "token");
    let claim = post(&app, "/runners/jobs/claim", None, json!({"token":token})).await;
    let job = text(&claim["job"], "id");
    let attempt = claim["job"]["attempts"].as_i64().unwrap();
    let payload: Value = serde_json::from_str(text(&claim["job"], "payload")).unwrap();
    assert_eq!(payload["script"][0], script);
    let output = "trace-content ".repeat(6000);
    assert!(output.len() > 64 * 1024);
    post(
        &app,
        &format!("/runners/jobs/{job}/trace"),
        None,
        json!({"token":token,"attempt":attempt,"output":output,"final":true}),
    )
    .await;
    let (status, trace) = request(
        &app,
        "GET",
        &format!("/projects/{project}/jobs/{job}/trace"),
        Some(&owner),
        json!({}),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(trace["data"]["output"], output);
    let completed = post(
        &app,
        &format!("/runners/jobs/{job}/complete"),
        None,
        json!({"token":token,"attempt":attempt,"result":output}),
    )
    .await;
    assert_eq!(completed["result"], output);
}
