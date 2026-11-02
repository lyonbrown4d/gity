use super::*;
use crate::gateway_headers::map_storage_location;
use crate::gateway_paths::{parse_request, target_uri};
use crate::{MirrorSettings, OciAuthorizationDecision, OciOperation, UpstreamPolicy};
use axum::http::{Method, StatusCode, Uri};
use std::{
  collections::{BTreeMap, BTreeSet},
  time::Duration,
};

#[test]
fn nested_repository_route_extracts_full_namespace_and_operation() {
  let uri: Uri = "/v2/acme/team/backend/manifests/v2.1".parse().unwrap();
  let (repository, operation, source, reference) = parse_request(&Method::GET, &uri).unwrap();
  assert_eq!(repository, "acme/team/backend");
  assert_eq!(operation, OciOperation::Pull);
  assert_eq!(source, None);
  assert_eq!(reference.as_deref(), Some("v2.1"));
}

#[test]
fn separates_registry_probe_from_disabled_catalog() {
  for path in ["/v2", "/v2/"] {
    let uri: Uri = path.parse().unwrap();
    let (repository, operation, source, reference) = parse_request(&Method::GET, &uri).unwrap();
    assert!(repository.is_empty());
    assert_eq!(operation, OciOperation::Probe);
    assert_eq!(source, None);
    assert_eq!(reference, None);
  }
  let catalog: Uri = "/v2/_catalog?n=10".parse().unwrap();
  assert_eq!(
    parse_request(&Method::GET, &catalog).unwrap().1,
    OciOperation::Catalog
  );
  assert_eq!(
    parse_request(&Method::POST, &"/v2".parse().unwrap())
      .unwrap_err()
      .0,
    StatusCode::METHOD_NOT_ALLOWED
  );
  assert_eq!(
    parse_request(&Method::POST, &"/v2/_catalog".parse().unwrap())
      .unwrap_err()
      .0,
    StatusCode::METHOD_NOT_ALLOWED
  );
}

#[test]
fn cross_repository_mount_extracts_source_repository() {
  let uri: Uri = "/v2/team/app/blobs/uploads/?mount=sha256%3Aabc&from=other%2Flibrary%2Fbase"
    .parse()
    .unwrap();
  let (repository, operation, source, reference) = parse_request(&Method::POST, &uri).unwrap();
  assert_eq!(repository, "team/app");
  assert_eq!(operation, OciOperation::Push);
  assert_eq!(source.as_deref(), Some("other/library/base"));
  assert_eq!(reference, None);
}

#[test]
fn storage_path_mapping_preserves_upload_state_query() {
  let decision = OciAuthorizationDecision {
    storage_repository: "project-9/repository".into(),
    mount_source_storage_repository: Some("project-2/base".into()),
    mirror_source: None,
  };
  let uri: Uri = "/v2/acme/repository/blobs/uploads/?from=acme%2Fbase&_state=token"
    .parse()
    .unwrap();
  let result = target_uri("http://127.0.0.1:18000", &uri, "acme/repository", &decision).unwrap();
  assert!(result.starts_with("http://127.0.0.1:18000/v2/project-9/repository/blobs/uploads/"));
  assert!(result.contains("from=project-2%2Fbase"));
  assert!(result.contains("_state=token"));
}

#[test]
fn mirror_sources_require_allowlisted_hosts_and_valid_mount_requests() {
  let settings = settings(Some(MirrorSettings {
    registries: BTreeMap::from([(
      "dockerhub".into(),
      crate::MirrorRegistry {
        host: "registry.example".into(),
        credential_secret_ref: None,
      },
    )]),
    allowed_hosts: BTreeSet::from(["registry.example".into()]),
  }));
  settings.validate().unwrap();
  let source: Uri = "/v2/team/repo/blobs/uploads/?mount=sha256%3Aabc"
    .parse()
    .unwrap();
  assert_eq!(
    parse_request(&Method::POST, &source).unwrap_err().0,
    StatusCode::BAD_REQUEST
  );
  assert!(
    settings
      .upstream_policy
      .allowed_hosts
      .contains("registry.example")
  );
}

#[test]
fn location_mapping_does_not_accept_other_repository_paths() {
  let result = map_storage_location(
    "/v2/project-10/repo/blobs/uploads/id",
    "project-1/repo",
    "team/repo",
  );
  assert_eq!(result.unwrap_err().0, StatusCode::BAD_GATEWAY);
}

fn settings(mirror: Option<MirrorSettings>) -> OciSettings {
  OciSettings {
    upstream: "http://127.0.0.1:18000".into(),
    request_timeout: Duration::from_secs(30),
    upstream_policy: UpstreamPolicy {
      allowed_hosts: BTreeSet::from(["registry.example".into()]),
    },
    mirror,
  }
}
