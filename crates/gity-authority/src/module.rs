use crate::AuthoritySettings;
use fluxdi::{
  Error as DiError, Injector, Provider, Shared,
  module::{Module, ModuleLifecycleFuture},
};
use gity_git_wire::identity_wire::{
  AuthorizeOciRequest, AuthorizeOciResponse, AuthorizePackageRequest, AuthorizePackageResponse,
  identity_authority_server::{IdentityAuthority, IdentityAuthorityServer},
};
use gity_oci::{AuthorizationError, OciAuthorization, OciRequest};
use std::{
  net::SocketAddr,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
};
use subtle::ConstantTimeEq;
use tokio::{
  net::TcpListener,
  sync::{Mutex, RwLock},
  task::JoinHandle,
};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;
use tonic::{
  Request, Response, Status,
  transport::{Certificate, Identity, Server, ServerTlsConfig},
};

const MAX_JSON_BYTES: usize = 1024 * 1024;

#[derive(Clone)]
pub struct AuthorityHealth {
  healthy: Arc<AtomicBool>,
  address: Arc<RwLock<Option<SocketAddr>>>,
}

impl AuthorityHealth {
  pub fn is_healthy(&self) -> bool {
    self.healthy.load(Ordering::Acquire)
  }

  pub async fn bound_address(&self) -> Option<SocketAddr> {
    *self.address.read().await
  }
}

#[derive(Clone)]
pub struct AuthorityModule {
  settings: AuthoritySettings,
  authorization: Arc<dyn OciAuthorization>,
  packages: Option<Arc<dyn gity_package_registry::PackageAuthorization>>,
}

impl AuthorityModule {
  pub fn with_packages(
    mut self,
    packages: Arc<dyn gity_package_registry::PackageAuthorization>,
  ) -> Self {
    self.packages = Some(packages);
    self
  }
  pub fn new(
    settings: AuthoritySettings,
    authorization: Arc<dyn OciAuthorization>,
  ) -> Result<Self, String> {
    settings.validate()?;
    Ok(Self {
      settings,
      authorization,
      packages: None,
    })
  }
}

struct AuthorityRuntime {
  settings: AuthoritySettings,
  authorization: Arc<dyn OciAuthorization>,
  health: AuthorityHealth,
  packages: Option<Arc<dyn gity_package_registry::PackageAuthorization>>,
  stop: CancellationToken,
  task: Mutex<Option<JoinHandle<Result<(), tonic::transport::Error>>>>,
}

impl Module for AuthorityModule {
  fn configure(&self, injector: &Injector) -> Result<(), DiError> {
    let health = AuthorityHealth {
      healthy: Arc::new(AtomicBool::new(false)),
      address: Arc::new(RwLock::new(None)),
    };
    let runtime_health = health.clone();
    injector.try_provide(Provider::<AuthorityHealth>::root(move |_| {
      Shared::new(health.clone())
    }))?;
    let settings = self.settings.clone();
    let authorization = self.authorization.clone();
    let packages = self.packages.clone();
    injector.try_provide(Provider::<AuthorityRuntime>::root(move |_| {
      Shared::new(AuthorityRuntime {
        settings: settings.clone(),
        authorization: authorization.clone(),
        packages: packages.clone(),
        health: runtime_health.clone(),
        stop: CancellationToken::new(),
        task: Mutex::new(None),
      })
    }))
  }

  fn on_start(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let runtime = injector.try_resolve::<AuthorityRuntime>()?;
      let listener = TcpListener::bind(runtime.settings.address)
        .await
        .map_err(|error| failure("on_start", error))?;
      *runtime.health.address.write().await = Some(
        listener
          .local_addr()
          .map_err(|error| failure("on_start", error))?,
      );
      gity_git_wire::configure_tls_crypto();
      let mut server = Server::builder();
      if let (Some(cert), Some(key), Some(ca)) = (
        &runtime.settings.tls_certificate,
        &runtime.settings.tls_key,
        &runtime.settings.client_ca,
      ) {
        let cert = tokio::fs::read(cert)
          .await
          .map_err(|error| failure("on_start", error))?;
        let key = tokio::fs::read(key)
          .await
          .map_err(|error| failure("on_start", error))?;
        let ca = tokio::fs::read(ca)
          .await
          .map_err(|error| failure("on_start", error))?;
        server = server
          .tls_config(
            ServerTlsConfig::new()
              .identity(Identity::from_pem(cert, key))
              .client_ca_root(Certificate::from_pem(ca)),
          )
          .map_err(|error| failure("on_start", error))?;
      }
      let rpc = OciAuthorityRpc {
        authorization: runtime.authorization.clone(),
        packages: runtime.packages.clone(),
      };
      let peer_token = runtime.settings.peer_token.clone();
      let service = IdentityAuthorityServer::with_interceptor(rpc, move |request: Request<()>| {
        authenticate_peer(&request, &peer_token).map(|()| request)
      });
      let stop = runtime.stop.clone();
      let health = runtime.health.clone();
      health.healthy.store(true, Ordering::Release);
      *runtime.task.lock().await = Some(tokio::spawn(async move {
        let result = server
          .add_service(service)
          .serve_with_incoming_shutdown(TcpListenerStream::new(listener), stop.cancelled_owned())
          .await;
        health.healthy.store(false, Ordering::Release);
        result
      }));
      Ok(())
    })
  }

  fn on_stop(&self, injector: Shared<Injector>) -> ModuleLifecycleFuture {
    Box::pin(async move {
      let runtime = injector.try_resolve::<AuthorityRuntime>()?;
      runtime.health.healthy.store(false, Ordering::Release);
      runtime.stop.cancel();
      if let Some(task) = runtime.task.lock().await.take() {
        task
          .await
          .map_err(|error| failure("on_stop", error))?
          .map_err(|error| failure("on_stop", error))?;
      }
      Ok(())
    })
  }
}

#[derive(Clone)]
struct OciAuthorityRpc {
  authorization: Arc<dyn OciAuthorization>,
  packages: Option<Arc<dyn gity_package_registry::PackageAuthorization>>,
}

#[tonic::async_trait]
impl IdentityAuthority for OciAuthorityRpc {
  async fn authorize_package(
    &self,
    request: Request<AuthorizePackageRequest>,
  ) -> Result<Response<AuthorizePackageResponse>, Status> {
    crate::package_rpc::authorize(self.packages.as_deref(), request.into_inner()).await
  }

  async fn authorize_oci(
    &self,
    request: Request<AuthorizeOciRequest>,
  ) -> Result<Response<AuthorizeOciResponse>, Status> {
    let request = request.into_inner();
    if request.request_json.len() > MAX_JSON_BYTES {
      return Err(Status::resource_exhausted(
        "OCI authorization request too large",
      ));
    }
    let context: OciRequest = serde_json::from_slice(&request.request_json)
      .map_err(|_| Status::invalid_argument("invalid OCI authorization request"))?;
    let decision = self
      .authorization
      .authorize(&context, request.authorization.as_deref())
      .await
      .map_err(map_authorization_error)?;
    let decision_json = serde_json::to_vec(&decision)
      .map_err(|_| Status::internal("could not encode OCI authorization result"))?;
    if decision_json.len() > MAX_JSON_BYTES {
      return Err(Status::resource_exhausted(
        "OCI authorization result too large",
      ));
    }
    Ok(Response::new(AuthorizeOciResponse { decision_json }))
  }
}

fn authenticate_peer<T>(request: &Request<T>, expected: &str) -> Result<(), Status> {
  let actual = request
    .metadata()
    .get("authorization")
    .and_then(|value| value.to_str().ok())
    .and_then(|value| value.strip_prefix("Bearer "))
    .unwrap_or_default();
  if actual.len() == expected.len() && bool::from(actual.as_bytes().ct_eq(expected.as_bytes())) {
    Ok(())
  } else {
    Err(Status::unauthenticated(
      "authority peer authentication failed",
    ))
  }
}

fn map_authorization_error(error: AuthorizationError) -> Status {
  match error {
    AuthorizationError::Unauthenticated => Status::unauthenticated("Gity credential rejected"),
    AuthorizationError::Forbidden => Status::permission_denied("operation forbidden"),
    AuthorizationError::NotFound => Status::not_found("repository not found"),
    AuthorizationError::Unavailable => Status::unavailable("authorization temporarily unavailable"),
  }
}

fn failure(phase: &str, error: impl std::fmt::Display) -> DiError {
  DiError::module_lifecycle_failed("AuthorityModule", phase, &error.to_string())
}
