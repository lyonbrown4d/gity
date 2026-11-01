use super::tests::*;
async fn lfs_req(
  s: &AppState,
  method: &str,
  path: &str,
  user: &str,
  body: Body,
) -> (StatusCode, Value, Vec<u8>) {
  let req = Request::builder()
    .method(method)
    .uri(path)
    .header(
      "authorization",
      format!("Basic {}", STANDARD.encode(format!("user:{user}"))),
    )
    .body(body)
    .unwrap();
  let response = handle_lfs(s.clone(), req)
    .await
    .unwrap_or_else(axum::response::IntoResponse::into_response);
  let status = response.status();
  let bytes = to_bytes(response.into_body(), 20 * 1024 * 1024)
    .await
    .unwrap()
    .to_vec();
  (
    status,
    serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    bytes,
  )
}
#[tokio::test]
async fn lfs_hash_verification_batch_streaming_and_lock_ownership() {
  let (_dir, s) = fixture().await;
  let oid = hex::encode(Sha256::digest(b"large object"));
  let path = format!("/team/p20.git/info/lfs/objects/{oid}");
  let upload = || Body::from("large object");
  assert_eq!(
    lfs_req(&s, "PUT", &path, "alice", Body::from("wrong"))
      .await
      .0,
    StatusCode::BAD_REQUEST
  );
  assert!(
    s.db
      .one(
        "SELECT id FROM project_lfs_objects WHERE project_id = ?",
        vec![20.into()]
      )
      .await
      .unwrap()
      .is_none()
  );
  assert_eq!(
    lfs_req(&s, "PUT", &path, "alice", upload()).await.0,
    StatusCode::OK
  );
  assert_eq!(
    lfs_req(&s, "GET", &path, "bob", Body::empty()).await.2,
    b"large object"
  );
  let batch = lfs_req(
    &s,
    "POST",
    "/team/p20.git/info/lfs/objects/batch",
    "bob",
    Body::from(
      json!({"operation":"download","objects":[{"oid":oid,"size":0},{"oid":"bad","size":0}]})
        .to_string(),
    ),
  )
  .await;
  assert_eq!(batch.0, StatusCode::OK);
  assert_eq!(batch.1["objects"][0]["size"], 12);
  assert_eq!(batch.1["objects"][1]["error"]["code"], 400);
  let lock = lfs_req(
    &s,
    "POST",
    "/team/p20.git/info/lfs/locks",
    "alice",
    Body::from(json!({"path":"binary/asset.psd"}).to_string()),
  )
  .await;
  assert_eq!(lock.0, StatusCode::CREATED);
  let id = lock.1["lock"]["id"].as_str().unwrap();
  let unlock = format!("/team/p20.git/info/lfs/locks/{id}/unlock");
  assert_eq!(
    lfs_req(&s, "POST", &unlock, "bob", Body::from("{}"))
      .await
      .0,
    StatusCode::FORBIDDEN
  );
  let verify = lfs_req(
    &s,
    "POST",
    "/team/p20.git/info/lfs/locks/verify",
    "bob",
    Body::from("{}"),
  )
  .await;
  assert_eq!(verify.1["ours"], json!([]));
  assert_eq!(verify.1["theirs"].as_array().unwrap().len(), 1);
  assert_eq!(
    lfs_req(&s, "POST", &unlock, "alice", Body::from("{}"))
      .await
      .0,
    StatusCode::OK
  );
  assert_eq!(
    lfs_req(&s, "GET", &path, "outsider", Body::empty()).await.0,
    StatusCode::FORBIDDEN
  );
}
