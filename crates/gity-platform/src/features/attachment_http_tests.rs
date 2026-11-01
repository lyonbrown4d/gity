use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;
#[tokio::test]
async fn attachment_and_wiki_access_are_scoped_and_actor_is_authoritative() {
  let (_dir, s) = fixture().await;
  request(
    &s,
    "POST",
    "/projects/20/issues",
    "alice",
    json!({"title":"Attach"}),
  )
  .await;
  let upload = request(
    &s,
    "POST",
    "/projects/20/issues/1/attachments",
    "bob",
    json!({"file_name":"../../note.txt","content_base64":"aGVsbG8=","uploaded_by_user_id":1}),
  )
  .await;
  assert_eq!(upload.0, StatusCode::OK);
  assert_eq!(upload.1["data"]["uploaded_by_user_id"], "2");
  let id = upload.1["data"]["id"].as_str().unwrap();
  assert_eq!(
    request(
      &s,
      "GET",
      &format!("/projects/20/issues/1/attachments/{id}"),
      "alice",
      json!({})
    )
    .await
    .1["data"]["content_base64"],
    "aGVsbG8="
  );
  assert_eq!(
    request(
      &s,
      "GET",
      &format!("/projects/30/issues/1/attachments/{id}"),
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
      "POST",
      "/projects/20/issues/1/attachments",
      "bob",
      json!({"file_name":"bad","content_base64":"!"})
    )
    .await
    .0,
    StatusCode::BAD_REQUEST
  );
  let wiki = request(
    &s,
    "POST",
    "/projects/20/wiki/pages",
    "alice",
    json!({"title":"Getting Started","content":"Read me","author_user_id":2}),
  )
  .await
  .1;
  assert_eq!(wiki["data"]["slug"], "getting-started");
  assert_eq!(wiki["data"]["author_user_id"], "1");
  assert_eq!(
    request(
      &s,
      "POST",
      "/projects/20/wiki/pages",
      "alice",
      json!({"title":"Getting Started"})
    )
    .await
    .0,
    StatusCode::CONFLICT
  );
  assert_eq!(
    request(
      &s,
      "GET",
      "/projects/30/wiki/pages/getting-started",
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
      "PATCH",
      "/projects/20/wiki/pages/getting-started",
      "bob",
      json!({"content":"Updated","editor_user_id":1})
    )
    .await
    .1["data"]["last_edited_by_user_id"],
    "2"
  );
}
