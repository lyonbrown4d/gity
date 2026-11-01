use super::*;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;
fn assert_references(value: &Value, document: &Value) {
    match value {
        Value::Object(map) => {
            if let Some(reference) = map.get("$ref").and_then(Value::as_str) {
                assert!(
                    reference.starts_with("#/"),
                    "external schema reference {reference}"
                );
                assert!(
                    document.pointer(&reference[1..]).is_some(),
                    "missing schema reference {reference}"
                );
            }
            for child in map.values() {
                assert_references(child, document);
            }
        }
        Value::Array(values) => {
            for child in values {
                assert_references(child, document);
            }
        }
        _ => {}
    }
}
#[test]
fn generated_contract_resolves_all_schemas_and_preserves_protocol_errors() {
    let spec = serde_json::to_value(document()).unwrap();
    let paths = spec["paths"].as_object().unwrap();
    assert!(paths.len() > 70, "nonempty coverage across domains");
    assert!(spec["components"]["securitySchemes"]["bearer"].is_object());
    let binary = &paths["/api/v1/projects/{id}/packages/generic/{name}/{version}/{file}"]["get"];
    for code in ["200", "206", "304", "412", "416", "401", "403", "404"] {
        assert!(
            binary["responses"].get(code).is_some(),
            "missing binary response {code}"
        );
    }
    assert!(
        paths["/api/v1/projects/{id}/issues/attachments"]["post"]["responses"]
            .get("201")
            .is_some()
    );
    assert!(
        paths["/{organization}/{project}/info/lfs/locks"]["post"]["responses"]
            .get("201")
            .is_some()
    );
    assert_eq!(
        paths["/api/v1/repos/{id}/issues"]["get"]["deprecated"],
        true
    );
    for (path, method) in [
        ("/api/health", "get"),
        ("/api/v1/rewrite/info", "get"),
        ("/api/v1/users/me", "get"),
        ("/api/v1/auth/login", "post"),
        ("/api/v1/orgs/{id}/members", "get"),
        ("/api/v1/projects/{id}/repository/branches", "get"),
        ("/api/v1/repos/{id}/branches", "post"),
        (
            "/api/v1/projects/{id}/repository/branch-protections/{branch}",
            "patch",
        ),
        ("/api/v1/projects/{id}/search/index/refresh", "post"),
        ("/api/v1/projects/{id}/file-commits", "post"),
        ("/{organization}/{project}/info/refs", "get"),
        ("/{organization}/{project}/git-receive-pack", "post"),
    ] {
        assert!(paths[path][method].is_object(), "missing {method} {path}");
    }
    let discovery =
        &paths["/{organization}/{project}/info/refs"]["get"]["responses"]["200"]["content"];
    assert!(
        discovery
            .get("application/x-git-upload-pack-advertisement")
            .is_some()
    );
    assert!(
        discovery
            .get("application/x-git-receive-pack-advertisement")
            .is_some()
    );
    let mut operation_ids = std::collections::HashSet::new();
    for (path, item) in paths {
        assert!(
            !path.contains("{*"),
            "OpenAPI catchall path must use standard variable syntax"
        );
        for method in ["get", "post", "put", "patch", "delete", "head"] {
            let Some(operation) = item.get(method) else {
                continue;
            };
            let id = operation["operationId"].as_str().unwrap();
            assert!(operation_ids.insert(id), "duplicate operation id {id}");
            let parameters = operation["parameters"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            for variable in path.split('{').skip(1).filter_map(|s| s.split('}').next()) {
                assert!(
                    parameters.iter().any(|v| v["name"] == variable
                        && v["in"] == "path"
                        && v["required"] == true),
                    "{method} {path} missing required parameter {variable}: {parameters:?}"
                );
            }
            assert!(!operation["responses"].as_object().unwrap().is_empty());
        }
    }
    assert_references(&spec, &spec);
}
#[tokio::test]
async fn swagger_serves_vendored_assets_and_raw_openapi_json_without_cdn() {
    let (_dir, state) = crate::features::test_support::fixture().await;
    let router = routes().with_state(state);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let spec: Value = serde_json::from_slice(
        &to_bytes(response.into_body(), 16 * 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(spec.get("openapi").is_some());
    assert!(spec.get("data").is_none());
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/docs/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let html = String::from_utf8(
        to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(html.contains("swagger-ui-bundle.js"));
    assert!(!html.contains("cdn.jsdelivr.net"));
    let response = router
        .oneshot(
            Request::builder()
                .uri("/docs/swagger-ui-bundle.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        to_bytes(response.into_body(), 16 * 1024 * 1024)
            .await
            .unwrap()
            .len()
            > 100_000
    );
}
