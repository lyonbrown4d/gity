use crate::{auth_tests::request, features::test_support::fixture};
use axum::http::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn directory_paginates_and_rejects_invalid_bounds() {
    let (_dir, state) = fixture().await;
    let first = request(
        &state,
        "GET",
        "/users?page=1&page_size=2",
        "alice",
        Value::Null,
    )
    .await;
    assert_eq!(first.0, StatusCode::OK);
    assert_eq!(first.1["data"]["total"], 3, "{}", first.1);
    assert_eq!(first.1["data"]["items"].as_array().unwrap().len(), 2);
    let next = request(
        &state,
        "GET",
        "/users?page=2&page_size=2",
        "alice",
        Value::Null,
    )
    .await;
    assert_eq!(next.1["data"]["items"][0]["username"], "outsider");
    for query in [
        "page=0",
        "page_size=0",
        "page_size=101",
        "page=18446744073709551615",
    ] {
        assert_eq!(
            request(
                &state,
                "GET",
                &format!("/users?{query}"),
                "alice",
                Value::Null
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
}

#[tokio::test]
async fn disabling_user_revokes_sessions_and_blocks_password_and_personal_tokens() {
    let (_dir, state) = fixture().await;
    state
        .db
        .execute("UPDATE users SET is_super_admin=1 WHERE id=1", vec![])
        .await
        .unwrap();
    super::set_password(&state, 2, "bob-password".into())
        .await
        .unwrap();
    let login = request(
        &state,
        "POST",
        "/auth/login",
        "",
        json!({"username":"bob","password":"bob-password"}),
    )
    .await;
    assert_eq!(login.0, StatusCode::OK);
    let token = login.1["data"]["token"].as_str().unwrap();
    let refresh = login.1["data"]["refresh_token"].as_str().unwrap();
    assert_eq!(
        request(
            &state,
            "PATCH",
            "/users/me",
            "bob",
            json!({"status":"disabled"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &state,
            "PATCH",
            "/users/2",
            "alice",
            json!({"status":"invalid"})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let disabled = request(
        &state,
        "PATCH",
        "/users/2",
        "alice",
        json!({"status":"disabled"}),
    )
    .await;
    assert_eq!(disabled.0, StatusCode::OK);
    assert_eq!(disabled.1["data"]["status"], "disabled");
    for credential in [token, "bob"] {
        assert_eq!(
            request(&state, "GET", "/users/me", credential, Value::Null)
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
    }
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
        request(
            &state,
            "POST",
            "/auth/login",
            "",
            json!({"username":"bob","password":"bob-password"})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(
            &state,
            "PATCH",
            "/users/2",
            "alice",
            json!({"status":"active"})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(&state, "GET", "/users/me", token, Value::Null)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(
            &state,
            "POST",
            "/auth/login",
            "",
            json!({"username":"bob","password":"bob-password"})
        )
        .await
        .0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn deleting_user_cleans_credentials_and_sessions_in_the_same_transaction() {
    let (_dir, state) = fixture().await;
    state
        .db
        .execute("UPDATE users SET is_super_admin=1 WHERE id=1", vec![])
        .await
        .unwrap();
    super::set_password(&state, 3, "delete-password".into())
        .await
        .unwrap();
    let login = request(
        &state,
        "POST",
        "/auth/login",
        "",
        json!({"username":"outsider","password":"delete-password"}),
    )
    .await;
    assert_eq!(login.0, StatusCode::OK);
    assert_eq!(
        request(&state, "DELETE", "/users/3", "alice", Value::Null)
            .await
            .0,
        StatusCode::OK
    );
    for table in ["auth_sessions", "user_passwords", "user_access_tokens"] {
        assert!(
            state
                .db
                .all(&format!("SELECT * FROM {table} WHERE user_id=3"), vec![])
                .await
                .unwrap()
                .is_empty(),
            "orphan data in {table}"
        );
    }
    assert_eq!(
        request(&state, "DELETE", "/users/3", "alice", Value::Null)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}
