use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;

async fn issue(s: &AppState, title: &str) -> i64 {
  let created = request(
    s,
    "POST",
    "/projects/20/issues",
    "alice",
    json!({"title": title}),
  )
  .await;
  assert_eq!(created.0, StatusCode::OK);
  created.1["data"]["number"].as_i64().unwrap()
}

async fn threads(s: &AppState, iid: i64, user: &str) -> Value {
  let path = format!("/projects/20/issues/{iid}/discussions");
  let response = request(s, "GET", &path, user, json!({})).await;
  assert_eq!(response.0, StatusCode::OK);
  response.1["data"].clone()
}

async fn reply(s: &AppState, iid: i64, user: &str, body: Value) -> (StatusCode, Value) {
  let path = format!("/projects/20/issues/{iid}/comments");
  request(s, "POST", &path, user, body).await
}

async fn resolve(
  s: &AppState,
  iid: i64,
  discussion: &str,
  user: &str,
  action: &str,
) -> (StatusCode, Value) {
  let path = format!("/projects/20/issues/{iid}/discussions/{discussion}/{action}");
  request(s, "PATCH", &path, user, json!({})).await
}

#[tokio::test]
async fn comments_open_threads_and_replies_join_the_same_thread() {
  let (_dir, s) = fixture().await;
  let iid = issue(&s, "Threads").await;
  assert_eq!(
    reply(&s, iid, "bob", json!({"body":"first"})).await.0,
    StatusCode::OK
  );
  let opened = threads(&s, iid, "bob").await;
  assert_eq!(opened.as_array().unwrap().len(), 1);
  assert_eq!(opened[0]["resolved"], false);
  assert!(opened[0]["resolved_by_user_id"].is_null());
  assert_eq!(opened[0]["comments"].as_array().unwrap().len(), 1);
  assert_eq!(opened[0]["comments"][0]["body"], "first");
  let discussion = opened[0]["id"].as_str().unwrap().to_owned();
  let joined = reply(
    &s,
    iid,
    "bob",
    json!({"body":"second","discussion_id":discussion}),
  )
  .await;
  assert_eq!(joined.0, StatusCode::OK);
  assert_eq!(joined.1["data"]["discussion_id"], discussion);
  let threaded = threads(&s, iid, "alice").await;
  assert_eq!(threaded.as_array().unwrap().len(), 1);
  let comments = threaded[0]["comments"].as_array().unwrap();
  assert_eq!(comments.len(), 2);
  assert_eq!(comments[0]["body"], "first");
  assert_eq!(comments[1]["body"], "second");
  // A top-level comment opens its own thread instead of joining the first one.
  assert_eq!(
    reply(&s, iid, "alice", json!({"body":"separate"})).await.0,
    StatusCode::OK
  );
  assert_eq!(
    reply(&s, iid, "bob", json!({"body":"third"})).await.0,
    StatusCode::OK
  );
  let ordered = threads(&s, iid, "alice").await;
  let bodies = ordered
    .as_array()
    .unwrap()
    .iter()
    .map(|thread| thread["comments"][0]["body"].as_str().unwrap().to_owned())
    .collect::<Vec<_>>();
  assert_eq!(bodies, ["first", "separate", "third"]);
}

#[tokio::test]
async fn replies_reject_discussions_of_another_issue_or_project() {
  let (_dir, s) = fixture().await;
  let first = issue(&s, "First").await;
  let second = issue(&s, "Second").await;
  assert_eq!(
    reply(&s, first, "alice", json!({"body":"anchor"})).await.0,
    StatusCode::OK
  );
  let discussion = threads(&s, first, "alice").await[0]["id"]
    .as_str()
    .unwrap()
    .to_owned();
  let foreign = reply(
    &s,
    second,
    "alice",
    json!({"body":"wrong issue","discussion_id":discussion}),
  )
  .await;
  assert_eq!(foreign.0, StatusCode::NOT_FOUND);
  let elsewhere = request(
    &s,
    "POST",
    "/projects/30/issues",
    "alice",
    json!({"title":"Elsewhere"}),
  )
  .await;
  assert_eq!(elsewhere.0, StatusCode::OK);
  let other_project = request(
    &s,
    "POST",
    "/projects/30/issues/1/comments",
    "alice",
    json!({"body":"wrong project","discussion_id":discussion}),
  )
  .await;
  assert_eq!(other_project.0, StatusCode::NOT_FOUND);
  assert_eq!(
    reply(&s, second, "alice", json!({"body":"own"})).await.0,
    StatusCode::OK
  );
  assert_eq!(
    threads(&s, second, "alice").await.as_array().unwrap().len(),
    1
  );
}

#[tokio::test]
async fn resolution_is_limited_to_the_author_or_a_writer() {
  let (_dir, s) = fixture().await;
  s.db
    .execute(
      "UPDATE project_members SET role='reporter' WHERE project_id=20 AND user_id=2",
      vec![],
    )
    .await
    .unwrap();
  let iid = issue(&s, "Resolution").await;
  assert_eq!(
    reply(&s, iid, "bob", json!({"body":"by bob"})).await.0,
    StatusCode::OK
  );
  let mine = threads(&s, iid, "bob").await[0]["id"]
    .as_str()
    .unwrap()
    .to_owned();
  let resolved = resolve(&s, iid, &mine, "bob", "resolve").await;
  assert_eq!(resolved.0, StatusCode::OK);
  assert_eq!(resolved.1["data"]["resolved"], true);
  assert_eq!(resolved.1["data"]["resolved_by_user_id"], "2");
  let reopened = resolve(&s, iid, &mine, "bob", "unresolve").await;
  assert_eq!(reopened.0, StatusCode::OK);
  assert_eq!(reopened.1["data"]["resolved"], false);
  assert!(reopened.1["data"]["resolved_by_user_id"].is_null());
  assert_eq!(
    reply(&s, iid, "alice", json!({"body":"by alice"})).await.0,
    StatusCode::OK
  );
  let theirs = threads(&s, iid, "alice").await[1]["id"]
    .as_str()
    .unwrap()
    .to_owned();
  // A reporter may comment but not resolve someone else's thread.
  assert_eq!(
    resolve(&s, iid, &theirs, "bob", "resolve").await.0,
    StatusCode::FORBIDDEN
  );
  let written = resolve(&s, iid, &theirs, "alice", "resolve").await;
  assert_eq!(written.0, StatusCode::OK);
  assert_eq!(written.1["data"]["resolved_by_user_id"], "1");
  assert_eq!(
    resolve(&s, iid, &theirs, "outsider", "resolve").await.0,
    StatusCode::FORBIDDEN
  );
}

#[tokio::test]
async fn inline_anchors_are_validated_and_persisted() {
  let (_dir, s) = fixture().await;
  let iid = issue(&s, "Anchors").await;
  let position = json!({"file":"src/main.rs","line":3,"column":7});
  assert_eq!(
    reply(&s, iid, "bob", json!({"body":"inline","position":position}))
      .await
      .0,
    StatusCode::OK
  );
  let stored = threads(&s, iid, "bob").await;
  assert_eq!(stored[0]["comments"][0]["position"], position);
  for invalid in [
    json!({"file":"src/main.rs","line":"three"}),
    json!({"file":"src/main.rs","column":2}),
    json!({"file":"","line":1}),
    json!("not-json"),
    json!(42),
  ] {
    assert_eq!(
      reply(&s, iid, "bob", json!({"body":"bad","position":invalid}))
        .await
        .0,
      StatusCode::BAD_REQUEST,
      "position {invalid} must be rejected"
    );
  }
  assert_eq!(threads(&s, iid, "bob").await.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn merge_request_comments_use_the_same_thread_model() {
  let (_dir, s) = fixture().await;
  s.db
    .insert(
      "project_merge_requests",
      fields(json!({
          "project_id":20,"iid":1,"author_user_id":1,"title":"Review",
          "state":"opened","source_branch":"feature","target_branch":"main",
      })),
    )
    .await
    .unwrap();
  let created = request(
    &s,
    "POST",
    "/projects/20/merge-requests/1/comments",
    "bob",
    json!({"body":"line note","position":{"file":"lib.rs","line":12}}),
  )
  .await;
  assert_eq!(created.0, StatusCode::OK);
  let threads = request(
    &s,
    "GET",
    "/projects/20/merge-requests/1/discussions",
    "alice",
    json!({}),
  )
  .await;
  assert_eq!(threads.0, StatusCode::OK);
  let list = &threads.1["data"];
  assert_eq!(list.as_array().unwrap().len(), 1);
  assert_eq!(list[0]["comments"][0]["position"]["line"], 12);
  let discussion = list[0]["id"].as_str().unwrap().to_owned();
  let resolved = request(
    &s,
    "PATCH",
    &format!("/projects/20/merge-requests/1/discussions/{discussion}/resolve"),
    "alice",
    json!({}),
  )
  .await;
  assert_eq!(resolved.0, StatusCode::OK);
  assert_eq!(resolved.1["data"]["resolved"], true);
}
