use crate::{auth_tests::request, features::test_support::fixture};
use axum::http::StatusCode;
use serde_json::{Value, json};

fn public_key() -> String {
  russh::keys::PrivateKey::random(&mut rand::rng(), russh::keys::Algorithm::Ed25519)
    .unwrap()
    .public_key()
    .to_openssh()
    .unwrap()
}

#[tokio::test]
async fn personal_key_api_canonicalizes_public_material_and_enforces_account_scope() {
  let (_dir, state) = fixture().await;
  let public = public_key();
  let created = request(
    &state,
    "POST",
    "/users/me/ssh-keys",
    "alice",
    json!({"title":" Laptop ","public_key":format!("{public} laptop-comment")}),
  )
  .await;
  assert_eq!(created.0, StatusCode::OK, "{}", created.1);
  let key = &created.1["data"];
  assert!(key["id"].is_string());
  assert_eq!(key["user_id"], "1");
  assert_eq!(key["title"], "Laptop");
  assert_eq!(key["public_key"], public);
  assert!(key["fingerprint"].as_str().unwrap().starts_with("SHA256:"));
  assert!(key["last_used_at"].is_null());
  assert!(key["created_at"].as_str().unwrap().ends_with('Z'));
  assert!(key.get("private_key").is_none());
  let duplicate = request(
    &state,
    "POST",
    "/users/me/ssh-keys",
    "bob",
    json!({"title":"Duplicate","public_key":public}),
  )
  .await;
  assert_eq!(duplicate.0, StatusCode::CONFLICT);
  assert_eq!(
    request(&state, "GET", "/users/me/ssh-keys", "bob", Value::Null)
      .await
      .1["data"],
    json!([])
  );
  let path = format!("/users/me/ssh-keys/{}", key["id"].as_str().unwrap());
  assert_eq!(
    request(&state, "DELETE", &path, "bob", Value::Null).await.0,
    StatusCode::NOT_FOUND
  );
  // Even a super administrator only controls their own SSH credentials.
  state
    .db
    .execute("UPDATE users SET is_super_admin=1 WHERE id=2", vec![])
    .await
    .unwrap();
  assert_eq!(
    request(&state, "DELETE", &path, "bob", Value::Null).await.0,
    StatusCode::NOT_FOUND
  );
  assert_eq!(
    request(&state, "GET", "/users/me/ssh-keys", "", Value::Null)
      .await
      .0,
    StatusCode::UNAUTHORIZED
  );
  assert_eq!(
    request(&state, "DELETE", &path, "alice", Value::Null)
      .await
      .0,
    StatusCode::OK
  );
  assert_eq!(
    request(&state, "DELETE", &path, "alice", Value::Null)
      .await
      .0,
    StatusCode::NOT_FOUND
  );
}

#[tokio::test]
async fn personal_keys_reject_private_options_invalid_and_cross_mode_keys() {
  let (_dir, state) = fixture().await;
  for raw in [
    "-----BEGIN OPENSSH PRIVATE KEY-----",
    "ssh-ed25519 invalid",
    "command=\"git\" ssh-ed25519 AAAA",
    "ssh-ed25519 AAAA\nssh-ed25519 AAAA",
  ] {
    let result = request(
      &state,
      "POST",
      "/users/me/ssh-keys",
      "alice",
      json!({"title":"Invalid","public_key":raw}),
    )
    .await;
    assert_eq!(result.0, StatusCode::BAD_REQUEST, "{}", result.1);
  }
  let public = public_key();
  let deploy = request(
    &state,
    "POST",
    "/projects/20/deploy-keys",
    "alice",
    json!({"title":"Deploy","public_key":public,"can_push":false}),
  )
  .await;
  assert_eq!(deploy.0, StatusCode::OK, "{}", deploy.1);
  let personal = request(
    &state,
    "POST",
    "/users/me/ssh-keys",
    "alice",
    json!({"title":"Personal","public_key":public}),
  )
  .await;
  assert_eq!(personal.0, StatusCode::CONFLICT);
  let public = public_key();
  assert_eq!(
    request(
      &state,
      "POST",
      "/users/me/ssh-keys",
      "alice",
      json!({"title":"Personal","public_key":public})
    )
    .await
    .0,
    StatusCode::OK
  );
  let deploy = request(
    &state,
    "POST",
    "/projects/20/deploy-keys",
    "alice",
    json!({"title":"Deploy","public_key":public,"can_push":false}),
  )
  .await;
  assert_eq!(deploy.0, StatusCode::CONFLICT);
}
