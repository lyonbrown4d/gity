//! OCI Distribution gateway primitives. The public gateway authenticates and
//! authorizes every request before forwarding it to the private Angos service.

mod authorization;
mod gateway;
mod gateway_headers;
mod gateway_paths;
mod mirror;
mod mirror_transport;
mod module;
mod oci_mirror_resolver;
mod settings;

pub use authorization::{
  AuthorizationError, MirrorSource, OciAuthorization, OciAuthorizationDecision, OciOperation,
  OciRequest,
};
pub use gateway::{OciGateway, router};
pub use mirror::{MirrorCredential, MirrorCredentialProvider, MirrorResolveError, MirrorResolver};
pub use module::{OciGatewayHealth, OciGatewayModule};
pub use oci_mirror_resolver::OciClientMirrorResolver;
pub use settings::{MirrorRegistry, MirrorSettings, OciSettings, UpstreamPolicy};
