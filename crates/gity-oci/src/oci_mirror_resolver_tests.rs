use super::*;
use std::{
  collections::{BTreeMap, BTreeSet},
  time::Duration,
};

struct NoCredentials;

#[async_trait]
impl MirrorCredentialProvider for NoCredentials {
  async fn resolve_basic(
    &self,
    _secret_ref: &str,
  ) -> Result<Option<MirrorCredential>, MirrorResolveError> {
    Ok(None)
  }
}

#[tokio::test]
#[ignore = "requires outbound HTTPS to Docker Hub"]
async fn docker_hub_bearer_flow_resolves_a_live_manifest_digest() {
  let settings = OciSettings {
    upstream: "http://127.0.0.1:1".into(),
    request_timeout: Duration::from_secs(30),
    upstream_policy: crate::UpstreamPolicy::default(),
    mirror: Some(MirrorSettings {
      registries: BTreeMap::from([(
        "dockerhub".into(),
        MirrorRegistry {
          host: "registry-1.docker.io".into(),
          credential_secret_ref: None,
        },
      )]),
      allowed_hosts: BTreeSet::from(["registry-1.docker.io".into(), "auth.docker.io".into()]),
    }),
  };
  let resolver = OciClientMirrorResolver::new(&settings, Arc::new(NoCredentials)).unwrap();
  let digest = resolver
    .resolve_digest(&MirrorSource {
      registry_alias: "dockerhub".into(),
      repository: "library/alpine".into(),
      reference: Some("latest".into()),
    })
    .await
    .unwrap();
  assert!(is_sha256_digest(&digest));
}

fn source(reference: &str) -> MirrorSource {
  MirrorSource {
    registry_alias: "dockerhub".into(),
    repository: "library/alpine".into(),
    reference: Some(reference.into()),
  }
}
fn registry() -> MirrorRegistry {
  MirrorRegistry {
    host: "registry-1.docker.io".into(),
    credential_secret_ref: None,
  }
}

#[test]
fn child_manifest_digest_is_an_immutable_oci_reference() {
  let digest = format!("sha256:{}", "a".repeat(64));
  let reference = upstream_reference(&source(&digest), &registry()).unwrap();
  assert_eq!(reference.digest(), Some(digest.as_str()));
  assert_eq!(reference.tag(), None);
  assert_eq!(
    reference.to_string(),
    format!("registry-1.docker.io/library/alpine@{digest}")
  );
}

#[test]
fn mutable_tag_keeps_its_existing_reference_semantics() {
  let reference = upstream_reference(&source("3.22"), &registry()).unwrap();
  assert_eq!(reference.tag(), Some("3.22"));
  assert_eq!(reference.digest(), None);
  assert_eq!(
    reference.to_string(),
    "registry-1.docker.io/library/alpine:3.22"
  );
}

#[test]
fn digest_validation_is_not_bypassed_by_typed_reference_construction() {
  for invalid in ["sha256:short", "sha256:not-hex", "unknown:aaaaaaaa"] {
    assert_eq!(
      upstream_reference(&source(invalid), &registry()).unwrap_err(),
      MirrorResolveError::InvalidResponse
    );
  }
}

#[test]
fn upstream_missing_manifests_preserve_not_found_including_referrer_fallback_tags() {
  use oci_client::errors::OciDistributionError as Error;
  for error in [
    Error::ImageManifestNotFoundError("sha256-fallback".into()),
    Error::ServerError {
      code: 404,
      url: "https://registry.example/v2/repo/manifests/sha256-fallback".into(),
      message: "missing".into(),
    },
  ] {
    assert_eq!(classify_oci_error(error), MirrorResolveError::NotFound);
  }
  for code in ["MANIFEST_UNKNOWN", "NAME_UNKNOWN", "NOT_FOUND"] {
    let envelope = serde_json::from_value(serde_json::json!({
      "errors":[{"code":code,"message":"missing","detail":null}]
    }))
    .unwrap();
    assert_eq!(
      classify_oci_error(Error::RegistryError {
        envelope,
        url: "https://registry.example".into()
      }),
      MirrorResolveError::NotFound
    );
  }
}

#[test]
fn upstream_auth_and_outage_are_not_reclassified_as_missing_manifests() {
  use oci_client::errors::OciDistributionError as Error;
  for (code, expected) in [
    (401, MirrorResolveError::Unauthenticated),
    (403, MirrorResolveError::Forbidden),
    (429, MirrorResolveError::Unavailable),
    (503, MirrorResolveError::Unavailable),
  ] {
    assert_eq!(
      classify_oci_error(Error::ServerError {
        code,
        url: "https://registry.example".into(),
        message: "upstream error".into()
      }),
      expected
    );
  }
}
