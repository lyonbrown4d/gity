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
}
