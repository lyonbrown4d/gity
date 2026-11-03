use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;
use tower::ServiceExt;

async fn create_issue(s: &AppState, confidential: bool) -> Value {
  request(
    s,
    "POST",
    "/projects/20/issues",
    "alice",
    json!({"title":if confidential {"secret marker"} else {"public marker"},"description":"sensitive body","confidential":confidential}),
  )
  .await
  .1["data"]
    .clone()
}

#[tokio::test]
async fn confidential_issues_hide_list_detail_children_and_raw_attachment_from_outsider() {
  let (_dir, s) = fixture().await;
  s.db
    .execute(
      "UPDATE projects SET visibility='public' WHERE id=20",
      vec![],
    )
    .await
    .unwrap();
  let public = create_issue(&s, false).await;
  let secret = create_issue(&s, true).await;
  let confidential_iid = secret["id"].as_str().unwrap();
  let comment = request(
    &s,
    "POST",
    &format!("/projects/20/issues/{confidential_iid}/comments"),
    "alice",
    json!({"body":"private comment marker"}),
  )
  .await;
  assert_eq!(comment.0, StatusCode::OK);
  let attachment = request(
    &s,
    "POST",
    &format!("/projects/20/issues/{confidential_iid}/attachments"),
    "alice",
    json!({"file_name":"secret.txt","content_base64":"cHJpdmF0ZSBmaWxl"}),
  )
  .await;
  assert_eq!(attachment.0, StatusCode::OK);
  let attachment_id = attachment.1["data"]["id"].as_str().unwrap();

  let listed = request(
    &s,
    "GET",
    "/projects/20/issues?page=1&page_size=100",
    "outsider",
    json!({}),
  )
  .await;
  assert_eq!(listed.0, StatusCode::OK);
  assert_eq!(listed.1["data"]["total"], 1);
  let items = listed.1["data"]["items"].as_array().unwrap();
  assert!(items.iter().any(|row| row["title"] == public["title"]));
  assert!(items.iter().all(|row| row["title"] != secret["title"]));

  for path in [
    format!("/projects/20/issues/{confidential_iid}"),
    format!("/projects/20/issues/{confidential_iid}/comments"),
    format!("/projects/20/issues/{confidential_iid}/timeline"),
    format!("/projects/20/issues/{confidential_iid}/attachments/{attachment_id}"),
    format!("/projects/20/issues/{confidential_iid}/attachments/{attachment_id}/raw"),
  ] {
    assert_eq!(
      request(&s, "GET", &path, "outsider", json!({})).await.0,
      StatusCode::NOT_FOUND,
      "{path}"
    );
  }
  let readable_public = request(
    &s,
    "GET",
    &format!("/projects/20/issues/{}", public["id"].as_str().unwrap()),
    "outsider",
    json!({}),
  )
  .await;
  assert_eq!(readable_public.0, StatusCode::OK);
}

#[tokio::test]
async fn confidential_access_tracks_membership_changes_for_existing_token() {
  let (_dir, s) = fixture().await;
  let secret = create_issue(&s, true).await;
  let iid = secret["id"].as_str().unwrap();
  let detail = format!("/projects/20/issues/{iid}");
  assert_eq!(
    request(&s, "GET", &detail, "bob", json!({})).await.0,
    StatusCode::OK
  );

  // Reporter is still project-readable and can see the confidential issue.
  s.db
    .execute(
      "UPDATE project_members SET role='reporter' WHERE project_id=20 AND user_id=2",
      vec![],
    )
    .await
    .unwrap();
  assert_eq!(
    request(&s, "GET", &detail, "bob", json!({})).await.0,
    StatusCode::OK
  );

  let assigned = request(
    &s,
    "PATCH",
    &format!("{detail}/assignees"),
    "alice",
    json!({"user_ids":["2"]}),
  )
  .await;
  assert_eq!(assigned.0, StatusCode::OK, "{}", assigned.1);

  // Guest can read the private project under its policy, but not this issue.
  s.db
    .execute(
      "UPDATE project_members SET role='guest' WHERE project_id=20 AND user_id=2",
      vec![],
    )
    .await
    .unwrap();
  let after_downgrade = request(&s, "GET", &detail, "bob", json!({})).await;
  assert_eq!(after_downgrade.0, StatusCode::OK);
  request(
    &s,
    "PATCH",
    &format!("{detail}/assignees"),
    "alice",
    json!({"user_ids":[]}),
  )
  .await;
  assert_eq!(
    request(&s, "GET", &detail, "bob", json!({})).await.0,
    StatusCode::NOT_FOUND
  );
  s.db
    .execute(
      "DELETE FROM project_members WHERE project_id=20 AND user_id=2",
      vec![],
    )
    .await
    .unwrap();
  assert_eq!(
    request(&s, "GET", &detail, "bob", json!({})).await.0,
    StatusCode::FORBIDDEN
  );
}

#[tokio::test]
async fn author_exception_requires_project_readability_and_project_tokens_do_not_inherit_it() {
  let (_dir, s) = fixture().await;
  let secret = create_issue(&s, true).await;
  let iid = secret["id"].as_str().unwrap();
  let detail = format!("/projects/20/issues/{iid}");
  let response = crate::router(s.clone())
    .oneshot(
      axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/projects/20/access-tokens")
        .header("authorization", "Bearer alice")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
          json!({"name":"read only","scopes":["read_api"]}).to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();
  let status = response.status();
  let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
    .await
    .unwrap();
  let token = (status, serde_json::from_slice::<Value>(&body).unwrap());
  assert_eq!(token.0, StatusCode::OK, "{}", token.1);
  let bearer = token.1["data"]["token"].as_str().unwrap().to_owned();

  // The author remains eligible after a role change only while the project is readable.
  s.db
    .execute(
      "UPDATE projects SET visibility='public' WHERE id=20",
      vec![],
    )
    .await
    .unwrap();
  s.db
    .execute(
      "UPDATE project_members SET role='guest' WHERE project_id=20 AND user_id=1",
      vec![],
    )
    .await
    .unwrap();
  assert_eq!(
    request(&s, "GET", &detail, "alice", json!({})).await.0,
    StatusCode::OK
  );
  s.db
    .execute(
      "DELETE FROM project_members WHERE project_id=20 AND user_id=1",
      vec![],
    )
    .await
    .unwrap();
  s.db
    .execute(
      "UPDATE projects SET visibility='private' WHERE id=20",
      vec![],
    )
    .await
    .unwrap();
  assert_eq!(
    request(&s, "GET", &detail, "alice", json!({})).await.0,
    StatusCode::FORBIDDEN
  );

  // Project tokens carry the creator's project scope, but never the creator's confidential exception.
  s.db
    .execute(
      "UPDATE projects SET visibility='public' WHERE id=20",
      vec![],
    )
    .await
    .unwrap();
  assert_eq!(
    request(&s, "GET", &detail, &bearer, json!({})).await.0,
    StatusCode::NOT_FOUND
  );
}
