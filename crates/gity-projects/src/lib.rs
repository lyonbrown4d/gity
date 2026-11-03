//! Project and repository access application services with typed persistence.
mod access;
mod audit;
mod credentials;
mod domain;
mod lfs;
mod membership;
mod module;
mod namespace;
mod organizations;
mod project;
mod protection;
mod queries;
mod search_queries;
mod search_scope;
pub use search_scope::SearchProject;
mod validation;
mod variables;

pub use audit::AuditRecord;
pub use domain::*;
pub use module::ProjectModule;

mod access_queries;
mod access_service;
mod project_service;
mod registry_settings;
mod repositories;
pub use access_service::RepositoryAccessService;
pub use project_service::ProjectService;
use repositories::{ProjectRepository, RepositoryAccessRepository};
