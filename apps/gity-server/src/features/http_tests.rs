use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;
#[tokio::test]
async fn issue_crud_legacy_contract_and_project_isolation() {
    let (_dir, s) = fixture().await;
    let (status, created) = request(
        &s,
        "POST",
        "/projects/20/issues",
        "alice",
        json!({"title":"Fix build","author_user_id":2}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(created["data"]["id"], "1");
    assert_eq!(created["data"]["repository_id"], "20");
    assert_eq!(created["data"]["author_user_id"], "1");
    assert_eq!(
        request(&s, "GET", "/repos/20/issues/by-number/1", "bob", json!({}))
            .await
            .1["data"],
        created["data"]
    );
    assert_eq!(
        request(&s, "GET", "/projects/30/issues/1", "alice", json!({}))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(&s, "GET", "/projects/20/issues", "outsider", json!({}))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &s,
            "POST",
            "/projects/20/issues/1/comments",
            "bob",
            json!({"content":"Review complete"})
        )
        .await
        .1["data"]["content"],
        "Review complete"
    );
    let labels = request(
        &s,
        "PATCH",
        "/projects/20/issues/1/labels",
        "alice",
        json!({"labels":[{"name":" bug ","color":"red"}],"names":["bug"]}),
    )
    .await
    .1;
    assert_eq!(labels["data"].as_array().unwrap().len(), 1);
    assert_eq!(labels["data"][0]["issue_id"], "1");
    request(
        &s,
        "PATCH",
        "/projects/20/issues/1/assignees",
        "alice",
        json!({"user_ids":[2]}),
    )
    .await;
    request(
        &s,
        "POST",
        "/projects/30/issues",
        "alice",
        json!({"title":"Other project's issue"}),
    )
    .await;
    request(
        &s,
        "PATCH",
        "/projects/30/issues/1/labels",
        "alice",
        json!({"names":["private-other-project-label"]}),
    )
    .await;
    request(
        &s,
        "PATCH",
        "/projects/30/issues/1/assignees",
        "alice",
        json!({"user_ids":[1]}),
    )
    .await;
    let detail = request(&s, "GET", "/projects/20/issues/1", "alice", json!({}))
        .await
        .1;
    assert_eq!(detail["data"]["id"], "1");
    assert_eq!(detail["data"]["assignee_user_ids"], json!(["2"]));
    assert_eq!(detail["data"]["assignee_user_id"], "2");
    assert_eq!(detail["data"]["assignees"][0]["issue_id"], "1");
    assert_eq!(detail["data"]["labels"], labels["data"]);
    let list = request(&s, "GET", "/projects/20/issues", "alice", json!({}))
        .await
        .1;
    assert_eq!(list["data"][0], detail["data"]);
    let other = request(&s, "GET", "/projects/30/issues", "alice", json!({}))
        .await
        .1;
    assert_eq!(other["data"][0]["assignee_user_ids"], json!(["1"]));
    assert_eq!(
        other["data"][0]["labels"][0]["name"],
        "private-other-project-label"
    );

    assert_eq!(
        request(
            &s,
            "PATCH",
            "/projects/20/issues/1/labels",
            "alice",
            json!({"names":["new",""]})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &s,
            "GET",
            "/projects/20/issues/1/labels",
            "alice",
            json!({})
        )
        .await
        .1["data"],
        labels["data"]
    );
    assert_eq!(
        request(
            &s,
            "PATCH",
            "/projects/20/issues/1",
            "alice",
            json!({"status":"closed"})
        )
        .await
        .1["data"]["status"],
        "closed"
    );
    assert_eq!(
        request(
            &s,
            "PATCH",
            "/projects/20/issues/1",
            "alice",
            json!({"state":"invalid"})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let todos = request(&s, "GET", "/todos?state=pending", "alice", json!({}))
        .await
        .1;
    let comment_todo = todos["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["kind"] == "issue_commented")
        .unwrap();
    let id = comment_todo["id"].as_str().unwrap();
    assert_eq!(
        request(&s, "POST", &format!("/todos/{id}/done"), "bob", json!({}))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(&s, "POST", &format!("/todos/{id}/done"), "alice", json!({}))
            .await
            .0,
        StatusCode::OK
    );
}
