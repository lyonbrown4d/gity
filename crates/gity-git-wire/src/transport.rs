use crate::{
  GitOperation, RepositoryKey,
  wire::{self, git_storage_client::GitStorageClient},
};
use serde::{Deserialize, Serialize};
use tonic::{
  Request,
  metadata::MetadataValue,
  transport::{Channel, ClientTlsConfig, Endpoint},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeEndpoint {
  pub id: String,
  pub uri: String,
  pub token: String,
  pub ca_certificate: Option<String>,
  pub client_certificate: Option<String>,
  pub client_key: Option<String>,
}
impl std::fmt::Debug for NodeEndpoint {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("NodeEndpoint")
      .field("id", &self.id)
      .field("uri", &self.uri)
      .finish_non_exhaustive()
  }
}
#[derive(Clone)]
pub struct NodeClient {
  endpoint: NodeEndpoint,
  channel: Channel,
  epoch: u64,
}
impl NodeClient {
  pub fn new(endpoint: NodeEndpoint) -> anyhow::Result<Self> {
    crate::configure_tls_crypto();
    anyhow::ensure!(
      !endpoint.token.is_empty(),
      "Git node credential is required"
    );
    let mut transport = Endpoint::from_shared(endpoint.uri.clone())?
      .connect_timeout(std::time::Duration::from_secs(10))
      .tcp_nodelay(true);
    if endpoint.uri.starts_with("https://") {
      let mut tls = ClientTlsConfig::new();
      if let Some(ca) = &endpoint.ca_certificate {
        tls = tls.ca_certificate(tonic::transport::Certificate::from_pem(ca));
      }
      if let (Some(cert), Some(key)) = (&endpoint.client_certificate, &endpoint.client_key) {
        tls = tls.identity(tonic::transport::Identity::from_pem(cert, key));
      }
      transport = transport.tls_config(tls)?;
    } else {
      let host = endpoint
        .uri
        .strip_prefix("http://")
        .unwrap_or_default()
        .split(':')
        .next()
        .unwrap_or_default();
      anyhow::ensure!(
        matches!(host, "127.0.0.1" | "localhost" | "[::1]"),
        "remote Git nodes require TLS"
      );
    }
    Ok(Self {
      endpoint,
      channel: transport.connect_lazy(),
      epoch: 1,
    })
  }
  pub fn with_epoch(mut self, epoch: u64) -> Self {
    self.epoch = epoch;
    self
  }
  pub fn epoch(&self) -> u64 {
    self.epoch
  }
  pub fn channel(&self) -> Channel {
    self.channel.clone()
  }
  /// Describe a replica source without delegating the caller's TLS private key.
  pub fn replication_endpoint(&self) -> NodeEndpoint {
    let mut endpoint = self.endpoint.clone();
    endpoint.client_certificate = None;
    endpoint.client_key = None;
    endpoint
  }
  pub fn endpoint(&self) -> &NodeEndpoint {
    &self.endpoint
  }
  pub fn id(&self) -> &str {
    &self.endpoint.id
  }
  pub fn client(&self) -> GitStorageClient<Channel> {
    GitStorageClient::new(self.channel.clone()).max_decoding_message_size(8 * 1024 * 1024)
  }
  pub fn authenticate<T>(&self, message: T) -> anyhow::Result<Request<T>> {
    let mut request = Request::new(message);
    let token = MetadataValue::try_from(format!("Bearer {}", self.endpoint.token))?;
    request.metadata_mut().insert("authorization", token);
    Ok(request)
  }
  pub async fn execute(
    &self,
    repository: &str,
    operation: GitOperation,
  ) -> anyhow::Result<serde_json::Value> {
    let key = RepositoryKey::try_from(repository.to_owned())?;
    let request = self.authenticate(wire::ExecuteRequest {
      repository_key: key.into(),
      operation_json: serde_json::to_vec(&operation)?,
      operation_id: gity_ids::new_operation_id().to_string(),
      epoch: self.epoch,
    })?;
    let response = self.client().execute(request).await?.into_inner();
    Ok(serde_json::from_slice(&response.result_json)?)
  }
  pub async fn health(&self) -> anyhow::Result<wire::HealthResponse> {
    Ok(
      self
        .client()
        .health(self.authenticate(wire::HealthRequest {})?)
        .await?
        .into_inner(),
    )
  }
}

/// Only a completed server rejection proves that an ambiguous transport did not
/// leave a writer running. Other failures keep replicas out of the read path.
pub fn definitive_rejection(error: &anyhow::Error) -> bool {
  error.downcast_ref::<tonic::Status>().is_some_and(|status| {
    matches!(
      status.code(),
      tonic::Code::InvalidArgument
        | tonic::Code::NotFound
        | tonic::Code::AlreadyExists
        | tonic::Code::PermissionDenied
        | tonic::Code::Unauthenticated
        | tonic::Code::FailedPrecondition
        | tonic::Code::Unimplemented
    )
  })
}
