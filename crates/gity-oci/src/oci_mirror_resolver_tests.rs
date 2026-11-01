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
