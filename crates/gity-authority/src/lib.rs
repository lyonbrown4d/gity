//! Private, peer-authenticated RPC forwarding for identity and OCI decisions.

mod module;
mod package_remote;
mod package_rpc;
mod remote;
pub use package_remote::RemotePackageAuthorization;
mod settings;

pub use module::{AuthorityHealth, AuthorityModule};
pub use remote::RemoteOciAuthorization;
pub use settings::AuthoritySettings;
