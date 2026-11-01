use super::merge_tests::file;
use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;
#[tokio::test]
async fn release_links_cannot_be_deleted_through_another_release() {
    let (_dir, s) = fixture().await;
    let repo = "20.git";
    s.git.init_bare(repo, "main").await.unwrap();
    s.git
        .create_file_commit(repo, file("main", "README.md", "Release"))
        .await
        .unwrap();
    let one = request(
        &s,
        "POST",
        "/projects/20/releases",
        "alice",
        json!({"tag_name":"v1","create_tag":true}),
    )
    .await;
    let two = request(
        &s,
        "POST",
        "/projects/20/releases",
        "alice",
        json!({"tag_name":"v2","create_tag":true}),
    )
    .await;
    assert_eq!(one.0, StatusCode::OK);
    assert_eq!(two.0, StatusCode::OK);
    let r1 = one.1["data"]["release"]["id"].as_str().unwrap();
    let r2 = two.1["data"]["release"]["id"].as_str().unwrap();
    let link = request(
        &s,
        "POST",
        &format!("/projects/20/releases/{r1}/links"),
        "alice",
        json!({"name":"Binary","url":"https://example.test/download"}),
    )
    .await
    .1;
    let id = link["data"]["id"].as_str().unwrap();
    assert_eq!(
        request(
            &s,
            "DELETE",
            &format!("/projects/20/releases/{r2}/links/{id}"),
            "alice",
            json!({})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(
            &s,
            "GET",
            &format!("/projects/20/releases/{r1}"),
            "alice",
            json!({})
        )
        .await
        .1["data"]["links"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        request(
            &s,
            "POST",
            &format!("/projects/20/releases/{r1}/links"),
            "alice",
            json!({"name":"Unsafe","url":"javascript:alert(1)"})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
}
