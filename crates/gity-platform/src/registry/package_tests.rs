use super::tests::*;
#[tokio::test]
async fn package_protocols_store_bytes_and_respect_conditional_range_and_tenant_scope() {
  let (_dir, s) = fixture().await;
  let path = "/projects/20/packages/generic/widgets/1.0.0/dist/widget.bin";
  let (status, _, bytes) = req(
    &s,
    "PUT",
    path,
    "alice",
    Body::from("abcdef"),
    &[("content-type", "application/octet-stream")],
  )
  .await;
  assert_eq!(status, StatusCode::OK);
  let created: Value = serde_json::from_slice(&bytes).unwrap();
  let id = created["data"]["id"].as_str().unwrap();
  let (status, headers, bytes) = req(&s, "GET", path, "bob", Body::empty(), &[]).await;
  assert_eq!(status, StatusCode::OK);
  assert_eq!(bytes, b"abcdef");
  let etag = headers["etag"].to_str().unwrap();
  let (status, headers, bytes) = req(
    &s,
    "GET",
    path,
    "bob",
    Body::empty(),
    &[("range", "bytes=2-4")],
  )
  .await;
  assert_eq!(status, StatusCode::PARTIAL_CONTENT);
  assert_eq!(headers["content-range"], "bytes 2-4/6");
  assert_eq!(bytes, b"cde");
  assert_eq!(
    req(
      &s,
      "GET",
      path,
      "bob",
      Body::empty(),
      &[("if-none-match", etag)]
    )
    .await
    .0,
    StatusCode::NOT_MODIFIED
  );
  assert_eq!(
    req(
      &s,
      "GET",
      path,
      "bob",
      Body::empty(),
      &[("if-match", "\"other\"")]
    )
    .await
    .0,
    StatusCode::PRECONDITION_FAILED
  );
  assert_eq!(
    req(
      &s,
      "GET",
      path,
      "bob",
      Body::empty(),
      &[("range", "bytes=99-")]
    )
    .await
    .0,
    StatusCode::RANGE_NOT_SATISFIABLE
  );
  assert_eq!(
    req(
      &s,
      "GET",
      path,
      "bob",
      Body::empty(),
      &[("range", "bytes=2-4"), ("if-range", "\"old\"")]
    )
    .await
    .2,
    b"abcdef"
  );
  assert_eq!(
    json_req(
      &s,
      "GET",
      &format!("/projects/30/packages/files/{id}"),
      "alice",
      json!({})
    )
    .await
    .0,
    StatusCode::NOT_FOUND
  );
  assert_eq!(
    req(&s, "GET", path, "outsider", Body::empty(), &[]).await.0,
    StatusCode::FORBIDDEN
  );
  let maven = "/projects/20/packages/maven/com/example/widget/2.0/widget-2.0.jar";
  assert_eq!(
    req(&s, "PUT", maven, "alice", Body::from("jar"), &[])
      .await
      .0,
    StatusCode::OK
  );
  assert_eq!(
    req(&s, "GET", maven, "bob", Body::empty(), &[]).await.2,
    b"jar"
  );
  let npm = json_req(
        &s,
        "PUT",
        "/projects/20/packages/npm/@scope/widget",
        "alice",
        json!({
            "name":"@scope/widget",
            "dist-tags":{"latest":"1.2.0"},
            "versions":{"1.2.0":{}},
            "_attachments":{"widget.tgz":{"data":STANDARD.encode("npm"),"content_type":"application/gzip"}},
        }),
    )
    .await;
  assert_eq!(npm.0, StatusCode::OK);
  assert_eq!(npm.1["dist-tags"]["latest"], "1.2.0");
  let pypi = "/projects/20/packages/pypi/lib/1.0/lib.whl";
  assert_eq!(
    req(&s, "PUT", pypi, "alice", Body::from("wheel"), &[])
      .await
      .0,
    StatusCode::OK
  );
  let simple = req(
    &s,
    "GET",
    "/projects/20/packages/pypi/simple/lib",
    "bob",
    Body::empty(),
    &[],
  )
  .await;
  assert_eq!(simple.0, StatusCode::OK);
  assert!(String::from_utf8(simple.2).unwrap().contains("lib.whl"));
  assert_eq!(
    req(
      &s,
      "GET",
      "/projects/20/packages/pypi/simple/lib/",
      "bob",
      Body::empty(),
      &[],
    )
    .await
    .0,
    StatusCode::OK
  );
  assert_eq!(
    req(
      &s,
      "GET",
      "/projects/20/packages/pypi/simple/",
      "bob",
      Body::empty(),
      &[],
    )
    .await
    .0,
    StatusCode::OK
  );
}

#[tokio::test]
async fn generic_upload_is_idempotent_for_equal_bytes_and_conflicts_on_replacement() {
  let (_dir, s) = fixture().await;
  let path = "/projects/20/packages/generic/widgets/1.0.0/dist/widget.bin";
  let first = req(&s, "PUT", path, "alice", Body::from("same"), &[]).await;
  assert_eq!(first.0, StatusCode::OK);
  let first: Value = serde_json::from_slice(&first.2).unwrap();
  let file_id = first["data"]["id"].as_str().unwrap().to_owned();

  let repeated = req(&s, "PUT", path, "alice", Body::from("same"), &[]).await;
  assert_eq!(repeated.0, StatusCode::OK);
  let repeated: Value = serde_json::from_slice(&repeated.2).unwrap();
  assert_eq!(repeated["data"]["id"], file_id);

  let replaced = req(&s, "PUT", path, "alice", Body::from("other"), &[]).await;
  assert_eq!(replaced.0, StatusCode::CONFLICT);
  let downloaded = req(&s, "GET", path, "bob", Body::empty(), &[]).await;
  assert_eq!(downloaded.2, b"same");
  assert_eq!(
    downloaded.1["x-checksum-sha256"],
    "0967115f2813a3541eaef77de9d9d5773f1c0c04314b0bbfe4ff3b3b1c55b5d5"
  );
}

#[tokio::test]
async fn concurrent_generic_retries_create_one_file_record() {
  let (_dir, s) = fixture().await;
  let path = "/projects/20/packages/generic/widgets/1.0.0/dist/concurrent.bin";
  let (left, right) = tokio::join!(
    req(&s, "PUT", path, "alice", Body::from("payload"), &[]),
    req(&s, "PUT", path, "alice", Body::from("payload"), &[]),
  );
  assert_eq!(left.0, StatusCode::OK);
  assert_eq!(right.0, StatusCode::OK);
  let rows = s.db.all(
        "SELECT f.id FROM project_package_files f JOIN project_package_versions v ON v.id=f.project_package_version_id WHERE v.version=? AND f.file_path=?",
        vec!["1.0.0".into(), "dist/concurrent.bin".into()],
    ).await.unwrap();
  assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn package_detail_keeps_envelope_and_string_id_contract() {
  let (_dir, state) = fixture().await;
  let path = "/projects/20/packages/generic/details/1.0.0/package.bin";
  assert_eq!(
    req(&state, "PUT", path, "alice", Body::from("bytes"), &[])
      .await
      .0,
    StatusCode::OK
  );
  let listed = json_req(
    &state,
    "GET",
    "/projects/20/packages?format=generic&name=details",
    "alice",
    json!({}),
  )
  .await;
  let package_id = listed.1["data"]["items"][0]["id"]
    .as_str()
    .unwrap()
    .to_owned();
  let detail = json_req(
    &state,
    "GET",
    &format!("/projects/20/packages/{package_id}"),
    "alice",
    json!({}),
  )
  .await;

  assert_eq!(detail.0, StatusCode::OK);
  assert_eq!(detail.1["code"], 0);
  assert_eq!(detail.1["data"]["package"]["id"], package_id);
  assert_eq!(
    detail.1["data"]["versions"][0]["version"]["version"],
    "1.0.0"
  );
}
