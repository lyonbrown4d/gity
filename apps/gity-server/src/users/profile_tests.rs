use crate::{auth_tests::request, features::test_support::fixture};
use axum::http::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn profile_password_change_is_atomic_revokes_sessions_and_keeps_personal_tokens() {
    let (_dir, state) = fixture().await;
    super::set_password(&state, 1, "old-password".into())
        .await
        .unwrap();
    let login = request(
        &state,
        "POST",
        "/auth/login",
        "",
        json!({"username":"alice","password":"old-password"}),
    )
    .await;
    assert_eq!(login.0, StatusCode::OK, "{}", login.1);
    let session = login.1["data"]["token"].as_str().unwrap();
    let refresh = login.1["data"]["refresh_token"].as_str().unwrap();
    let invalid = request(&state, "PATCH", "/users/me", session, json!({"username":"must-not-persist","email":"must-not-persist@example.test","password":"short"})).await;
    assert_eq!(invalid.0, StatusCode::BAD_REQUEST);
    let unchanged = request(&state, "GET", "/users/me", session, Value::Null).await;
    assert_eq!(unchanged.0, StatusCode::OK);
    assert_eq!(unchanged.1["data"]["username"], "alice");
    assert_eq!(unchanged.1["data"]["email"], "alice@test.local");
    let before = state
        .db
        .one(
            "SELECT password_hash FROM user_passwords WHERE user_id=1",
            vec![],
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        request(
            &state,
            "PATCH",
            "/users/me",
            session,
            json!({"email":"blank@example.test","password":"   "})
        )
        .await
        .0,
        StatusCode::OK
    );
    let after = state
        .db
        .one(
            "SELECT password_hash FROM user_passwords WHERE user_id=1",
            vec![],
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(before, after);
    assert_eq!(
        request(&state, "GET", "/users/me", session, Value::Null)
            .await
            .0,
        StatusCode::OK
    );
    let changed = request(&state, "PATCH", "/users/me", session, json!({"username":"alice.renamed","email":"changed@example.test","password":"new-password"})).await;
    assert_eq!(changed.0, StatusCode::OK, "{}", changed.1);
    assert_eq!(changed.1["data"]["username"], "alice.renamed");
    assert_eq!(changed.1["data"]["email"], "changed@example.test");
    assert!(changed.1["data"].get("password").is_none());
    assert!(changed.1["data"].get("password_hash").is_none());
    assert_eq!(
        request(&state, "GET", "/users/me", session, Value::Null)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/auth/refresh",
            "",
            json!({"refresh_token":refresh})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(&state, "GET", "/users/me", "alice", Value::Null)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/auth/login",
            "",
            json!({"username":"alice.renamed","password":"old-password"})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let new_login = request(
        &state,
        "POST",
        "/auth/login",
        "",
        json!({"username":"alice.renamed","password":"new-password"}),
    )
    .await;
    assert_eq!(new_login.0, StatusCode::OK);
    let new_session = new_login.1["data"]["token"].as_str().unwrap();
    assert_eq!(
        request(
            &state,
            "PATCH",
            "/users/me",
            new_session,
            json!({"username":"bob","password":"another-password"})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/auth/login",
            "",
            json!({"username":"alice.renamed","password":"new-password"})
        )
        .await
        .0,
        StatusCode::OK
    );
}
