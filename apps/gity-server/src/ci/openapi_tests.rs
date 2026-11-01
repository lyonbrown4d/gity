use super::*;
#[test]
fn ci_openapi_has_typed_requests_auth_errors_and_binary_source() {
    let (_, document) = documented_routes().split_for_parts();
    let document = serde_json::to_value(document).unwrap();
    for path in [
        "/projects/{project}/pipelines",
        "/repos/{project}/pipelines",
        "/projects/{project}/jobs/{job}/retry",
        "/runners/jobs/claim",
        "/runners/jobs/{job}/source",
    ] {
        assert!(document["paths"].get(path).is_some(), "{path}");
    }
    let trace = &document["paths"]["/runners/jobs/{job}/trace"]["post"];
    assert_eq!(
        trace["requestBody"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/CiRunnerAppendTrace"
    );
    for status in ["401", "403", "404", "409", "413", "422", "500"] {
        assert!(trace["responses"].get(status).is_some(), "{status}");
    }
    let source = &document["paths"]["/runners/jobs/{job}/source"]["post"];
    assert!(
        source["responses"]["200"]["content"]
            .get("application/zip")
            .is_some()
    );
    let token = &document["components"]["schemas"]["CiRunnerToken"];
    assert_eq!(token["properties"]["token"]["writeOnly"], true);
    let artifact = &document["components"]["schemas"]["CiJobArtifact"];
    assert!(artifact["properties"].get("storage_key").is_none());
}
