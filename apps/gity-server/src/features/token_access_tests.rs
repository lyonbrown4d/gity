use super::test_support::fixture;
use crate::AppState;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use base64::Engine;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn request(
    s: &AppState,
    method: &str,
    path: &str,
    token: &str,
    body: Value,
) -> (StatusCode, Value) {
    let response = crate::router(s.clone())
        .oneshot(
            Request::builder()
                .method(method)
                .uri(format!("/api/v1{path}"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let value = serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap())
        .unwrap();
    (status, value)
}
async fn token(s: &AppState, scope: &str) -> String {
    let (status, value) = request(
        s,
        "POST",
        "/projects/20/access-tokens",
        "alice",
        json!({"name":scope,"scopes":[scope]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value["data"]["token"].as_str().unwrap().into()
}

#[tokio::test]
async fn api_scopes_preserve_legacy_actions_without_inheriting_creator_privileges() {
    let (_dir, state) = fixture().await;
    state
        .db
        .update(
            "users",
            1,
            json!({"is_super_admin":1}).as_object().unwrap().clone(),
        )
        .await
        .unwrap();
    let read = token(&state, "read_api").await;
    let write = token(&state, "write_api").await;
    let repository = token(&state, "write_repository").await;
    state.git.init_bare("20.git", "main").await.unwrap();
    state
        .git
        .create_file_commit(
            "20.git",
            super::merge_tests::file("main", "README.md", "Base"),
        )
        .await
        .unwrap();
    state
        .git
        .create_branch("20.git", "feature", "main")
        .await
        .unwrap();
    let issue = request(
        &state,
        "POST",
        "/projects/20/issues",
        &read,
        json!({"title":"Scoped issue","author_user_id":2}),
    )
    .await;
    assert_eq!(issue.0, StatusCode::OK, "{}", issue.1);
    assert_eq!(issue.1["data"]["author_user_id"], "1");
    assert_eq!(
        request(
            &state,
            "PATCH",
            "/projects/20/issues/1",
            &read,
            json!({"title":"Changed"})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/projects/30/issues",
            &read,
            json!({"title":"Wrong project"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/projects/20/issues",
            &repository,
            json!({"title":"Wrong scope"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let page = json!({"title":"Scoped wiki","slug":"scoped","content":"Written by write_api"});
    assert_eq!(
        request(
            &state,
            "POST",
            "/projects/20/wiki/pages",
            &read,
            page.clone()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(&state, "POST", "/projects/20/wiki/pages", &write, page)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "GET",
            "/projects/20/wiki/pages/scoped",
            &read,
            Value::Null
        )
        .await
        .1["data"]["content"],
        "Written by write_api"
    );
    let permissions = request(
        &state,
        "GET",
        "/projects/20/permissions",
        &read,
        Value::Null,
    )
    .await
    .1;
    assert_eq!(permissions["data"]["role"], "project_token");
    assert_eq!(permissions["data"]["role_level"], 0);
    assert_eq!(permissions["data"]["actions"]["project.issues.write"], true);
    assert_eq!(
        permissions["data"]["actions"]["project.repository.push"],
        false
    );
    assert_eq!(
        permissions["data"]["actions"]["project.merge_requests.merge"],
        false
    );
    assert_eq!(
        request(&state, "GET", "/projects/20", &read, Value::Null)
            .await
            .0,
        StatusCode::OK
    );
    let file = json!({"branch_name":"feature","file_path":"src.txt","content":"Scoped commit","message":"Scoped"});
    assert_eq!(
        request(
            &state,
            "POST",
            "/projects/20/repository/files",
            &write,
            file.clone()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/projects/20/repository/files",
            &repository,
            file
        )
        .await
        .0,
        StatusCode::OK
    );
    let mr = request(
        &state,
        "POST",
        "/projects/20/merge-requests",
        &read,
        json!({"title":"Scoped MR","source_branch":"feature","target_branch":"main"}),
    )
    .await;
    assert_eq!(mr.0, StatusCode::OK, "{}", mr.1);
    assert_eq!(
        request(
            &state,
            "POST",
            "/projects/20/merge-requests/1/comments",
            &read,
            json!({"body":"Scoped comment"})
        )
        .await
        .0,
        StatusCode::OK
    );
    for (method, path) in [
        ("POST", "/projects/20/merge-requests/1/merge"),
        ("POST", "/projects/20/releases"),
        ("POST", "/projects/20/access-tokens"),
        ("POST", "/projects/20/repository/branches/main/protect"),
        ("DELETE", "/projects/20/repository/branches/feature"),
    ] {
        assert_eq!(
            request(&state, method, path, &write, json!({})).await.0,
            StatusCode::FORBIDDEN,
            "{path}"
        );
    }
    for path in ["/users/me", "/orgs", "/todos", "/projects"] {
        assert_eq!(
            request(&state, "GET", path, &write, Value::Null).await.0,
            StatusCode::UNAUTHORIZED,
            "{path}"
        );
    }
    for (name, value) in [
        ("private-token", read.clone()),
        (
            "authorization",
            format!(
                "Basic {}",
                base64::engine::general_purpose::STANDARD.encode(format!("project:{read}"))
            ),
        ),
    ] {
        let response = crate::router(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/20/issues")
                    .header(name, value)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    assert!(
        state
            .db
            .one("SELECT id FROM project_issues WHERE project_id=30", vec![])
            .await
            .unwrap()
            .is_none()
    );
}
