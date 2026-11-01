use gity_git_wire::{GitOperation, NodeClient, NodeEndpoint};

#[tokio::test]
async fn replication_endpoint_keeps_connection_settings_but_drops_caller_identity() {
  let caller = NodeClient::new(NodeEndpoint {
    id: "git-1".into(),
    uri: "http://127.0.0.1:9101".into(),
    token: "peer-token-test-only".into(),
    ca_certificate: Some("test-ca".into()),
    client_certificate: Some("caller-certificate".into()),
    client_key: Some("caller-private-key".into()),
  })
  .unwrap();

  let source = caller.replication_endpoint();
  assert_eq!(source.id, "git-1");
  assert_eq!(source.uri, "http://127.0.0.1:9101");
  assert_eq!(source.token, "peer-token-test-only");
  assert_eq!(source.ca_certificate.as_deref(), Some("test-ca"));
  assert!(source.client_certificate.is_none());
  assert!(source.client_key.is_none());
  assert_eq!(
    caller.endpoint().client_key.as_deref(),
    Some("caller-private-key")
  );

  let operation = GitOperation::SyncReplica {
    source,
    source_epoch: 7,
  };
  let wire = serde_json::to_value(operation).unwrap();
  assert!(wire["source"]["client_certificate"].is_null());
  assert!(wire["source"]["client_key"].is_null());
}
