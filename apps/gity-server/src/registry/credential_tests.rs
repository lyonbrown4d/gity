use super::tests::*;
#[tokio::test]
async fn tokens_return_secret_once_enforce_project_scope_and_revoke() {
    let (_dir, s) = fixture().await;
    let (status, created) = json_req(
        &s,
        "POST",
        "/projects/20/deploy-tokens",
        "alice",
        json!({"name":"Publisher","scopes":["write_package"],"created_by_user_id":2}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let raw = created["data"]["token"].as_str().unwrap();
    assert_eq!(created["data"]["project_token"]["created_by_user_id"], "1");
    let row =
        s.db.one(
            "SELECT * FROM project_access_tokens WHERE id = ?",
            vec![
                created["data"]["project_token"]["id"]
                    .as_str()
                    .unwrap()
                    .parse::<i64>()
                    .unwrap()
                    .into(),
            ],
        )
        .await
        .unwrap()
        .unwrap();
    assert_ne!(text(&row, "token"), raw);
    assert!(text(&row, "token").starts_with("sha256:"));
    assert!(
        authenticate_token(&s, text(&row, "token"))
            .await
            .unwrap()
            .is_none()
    );
    let listed = json_req(&s, "GET", "/projects/20/deploy-tokens", "alice", json!({}))
        .await
        .1;
    assert!(listed["data"][0].get("token").is_none());
    assert_eq!(
        json_req(&s, "GET", "/projects/20/packages", raw, json!({}))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        json_req(&s, "GET", "/projects/30/packages", raw, json!({}))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        json_req(
            &s,
            "POST",
            "/projects/20/access-tokens",
            raw,
            json!({"name":"Escalate","scopes":["write_api"]})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let id = created["data"]["project_token"]["id"].as_str().unwrap();
    assert_eq!(
        json_req(
            &s,
            "DELETE",
            &format!("/projects/20/deploy-tokens/{id}"),
            "alice",
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        json_req(&s, "GET", "/projects/20/packages", raw, json!({}))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
}
#[tokio::test]
async fn token_expiry_and_masked_variable_updates_follow_persisted_utc_values() {
    let (_dir, s) = fixture().await;
    let (status, created) = json_req(
        &s,
        "POST",
        "/projects/20/access-tokens",
        "alice",
        json!({
            "name":"Expiring","scopes":["read_package"],"expires_at":"2000-01-01T00:00:00Z"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let raw = created["data"]["token"].as_str().unwrap();
    assert_eq!(created["data"]["project_token"]["active"], false);
    assert_eq!(
        json_req(&s, "GET", "/projects/20/packages", raw, json!({}))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let first = json_req(
        &s,
        "PATCH",
        "/projects/20/ci/variables",
        "alice",
        json!({"key":"build_token","value":"secret-value","masked":true,"protected":true}),
    )
    .await;
    assert_eq!(first.0, StatusCode::OK);
    assert_eq!(first.1["data"]["key"], "BUILD_TOKEN");
    assert_eq!(first.1["data"]["value"], "");
    let second = json_req(
        &s,
        "PATCH",
        "/projects/20/ci/variables",
        "alice",
        json!({"key":"BUILD_TOKEN","value":"changed-secret","masked":true}),
    )
    .await;
    assert_eq!(second.1["data"]["id"], first.1["data"]["id"]);
    let stored =
        s.db.one(
            "SELECT * FROM project_ci_variables WHERE project_id = ?",
            vec![20.into()],
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored["value"], "changed-secret");
    assert_eq!(
        json_req(&s, "GET", "/projects/20/ci/variables", "bob", json!({}))
            .await
            .1["data"][0]["value"],
        ""
    );
    assert_eq!(
        json_req(
            &s,
            "PATCH",
            "/projects/20/ci/variables",
            "alice",
            json!({"key":"BUILD_TOKEN","value":"new\nsecret","masked":true})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        json_req(
            &s,
            "GET",
            "/projects/20/ci/variables",
            "outsider",
            json!({})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}
