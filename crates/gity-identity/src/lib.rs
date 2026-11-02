//! Identity and authorization domain services backed by typed SeaORM entities.
mod domain;
mod module;
mod oauth;
mod oauth_module;
mod repository;

pub use domain::{
  MembershipRole, PersonalSshKeyCommand, ProjectAccess, UserIdentity, UserProfileChanges,
};
pub use module::IdentityModule;
pub use oauth::{
  AuthorizationRequest, ClientRegistration, OAuthError, OAuthPrincipal, OAuthService,
  PendingAuthorization, TokenReply, TokenRequest,
};
pub use oauth_module::OAuthModule;
pub use repository::{
  BootstrapAccount, BootstrapAccountCommand, BootstrapGuard, IdentityRepository,
};
