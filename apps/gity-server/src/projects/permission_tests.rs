use crate::features::test_support::fixture;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn get(state: &crate::AppState, path: &str, user: &str) -> (StatusCode, Value) {
    let response = crate::router(state.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/{path}/20/permissions"))
                .header("authorization", format!("Bearer {user}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let value = serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap())
        .unwrap();
    (status, value)
}

#[tokio::test]
async fn effective_role_and_legacy_actions_match_capabilities_without_granting_access() {
    let (_dir, state) = fixture().await;
    let member = state
        .db
        .one(
            "SELECT id FROM project_members WHERE project_id=? AND user_id=?",
            vec![20.into(), 1.into()],
        )
        .await
        .unwrap()
        .unwrap();
    let member_id = member["id"].as_i64().unwrap();
    for (role, level) in [
        ("guest", 10),
        ("reporter", 20),
        ("developer", 30),
        ("maintainer", 40),
        ("owner", 50),
    ] {
        state
            .db
            .update(
                "project_members",
                member_id,
                json!({"role":role}).as_object().unwrap().clone(),
            )
            .await
            .unwrap();
        let (status, response) = get(&state, "projects", "alice").await;
        assert_eq!(status, StatusCode::OK);
        let data = &response["data"];
        assert_eq!(data["project_id"], "20");
        assert_eq!(data["user_id"], "1");
        assert_eq!(data["role"], role);
        assert_eq!(data["role_level"], level);
        assert_eq!(data["actions"].as_object().unwrap().len(), 21);
        for (action, capability, minimum) in [
            ("project.read", "can_read", 0),
            ("project.write", "can_write", 30),
            ("project.delete", "can_delete", 50),
            ("project.issues.create", "issue_create", 20),
            ("project.merge_requests.merge", "merge_request_merge", 40),
            ("project.jobs.read", "ci_read", 20),
            ("project.runners.admin", "runner_admin", 40),
        ] {
            assert_eq!(data["actions"][action], level >= minimum, "{role} {action}");
            assert_eq!(data["capabilities"][capability], data["actions"][action]);
        }
        assert_eq!(get(&state, "repos", "alice").await.1, response);
    }
    assert_eq!(
        get(&state, "projects", "outsider").await.0,
        StatusCode::FORBIDDEN
    );
    state
        .db
        .insert(
            "organization_members",
            json!({"organization_id":10,"user_id":2,"role":"owner"})
                .as_object()
                .unwrap()
                .clone(),
        )
        .await
        .unwrap();
    assert_eq!(
        get(&state, "projects", "bob").await.1["data"]["role"],
        "owner"
    );
    state
        .db
        .update(
            "projects",
            20,
            json!({"visibility":"public"}).as_object().unwrap().clone(),
        )
        .await
        .unwrap();
    let public = get(&state, "projects", "outsider").await.1;
    assert_eq!(public["data"]["role_level"], 0);
    assert_eq!(public["data"]["actions"]["project.read"], true);
    assert_eq!(public["data"]["actions"]["project.jobs.read"], false);
    assert_eq!(public["data"]["capabilities"]["can_write"], false);
    state
        .db
        .update(
            "users",
            3,
            json!({"is_super_admin":1}).as_object().unwrap().clone(),
        )
        .await
        .unwrap();
    let admin = get(&state, "projects", "outsider").await.1;
    assert_eq!(admin["data"]["role_level"], 100);
    assert!(
        admin["data"]["actions"]
            .as_object()
            .unwrap()
            .values()
            .all(|v| v == true)
    );
}
