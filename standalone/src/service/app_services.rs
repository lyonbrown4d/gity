use crate::configuration::cfg::Config;
use crate::service::auth_service::AuthService;
use crate::service::git_backend_service::GitBackendService;
use crate::service::organization_service::OrganizationService;
use crate::service::repository_service::RepositoryService;
use crate::service::user_service::UserService;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppServices {
  pub auth: AuthService,
  pub git_backend: GitBackendService,
  pub organization: OrganizationService,
  pub repository: RepositoryService,
  pub user: UserService,
}

impl AppServices {
  pub fn new(config: &Config, db_conn: DatabaseConnection) -> Self {
    let git_backend = GitBackendService::new(config, db_conn.clone());
    let user = UserService::new(config, db_conn.clone());
    Self {
      auth: AuthService::new(config, db_conn.clone()),
      organization: OrganizationService::new(config, db_conn.clone()),
      repository: RepositoryService::new(config, db_conn, git_backend.clone()),
      user,
      git_backend,
    }
  }
}
