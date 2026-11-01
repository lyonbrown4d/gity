//! Internal Git storage protocol. Public transports authorize before invoking it.
mod operations;
mod snapshot;
pub use snapshot::{ReferenceSnapshot, RepositoryHead};
mod router;
mod transport;
pub use operations::*;
pub use router::{FixedNodeRouter, GitNodeRouter};
pub use transport::{NodeClient, NodeEndpoint, definitive_rejection};
pub mod wire {
  tonic::include_proto!("gity.git.v1");
}

pub mod identity_wire {
  tonic::include_proto!("gity.identity.v1");
}

/// Select an RPC TLS provider even when HTTP and SQL dependencies enable another.
/// An application-selected provider takes precedence over this library default.
pub fn configure_tls_crypto() {
  let _ = rustls::crypto::ring::default_provider().install_default();
}
