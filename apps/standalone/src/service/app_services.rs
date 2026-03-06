use crate::configuration::cfg::Config;
use crate::service::auth_service::AuthService;
use crate::service::git_backend_service::GitBackendService;
use crate::service::organization_service::OrganizationService;
use crate::service::repository_service::RepositoryService;
use crate::service::user_service::UserService;
use repository::{
  OrganizationMembersRepository, OrganizationsRepository, RepositoriesRepository,
  RepositoryBranchesRepository, UsersRepository,
};
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
    let organizations_repository = OrganizationsRepository::new(db_conn.clone());
    let organization_members_repository = OrganizationMembersRepository::new(db_conn.clone());
    let repositories_repository = RepositoriesRepository::new(db_conn.clone());
    let repository_branches_repository = RepositoryBranchesRepository::new(db_conn.clone());
    let users_repository = UsersRepository::new(db_conn);

    let git_backend = GitBackendService::new(config, repository_branches_repository);
    let user = UserService::new(config, users_repository.clone());
    Self {
      auth: AuthService::new(
        config,
        users_repository.clone(),
        organizations_repository.clone(),
        organization_members_repository.clone(),
      ),
      organization: OrganizationService::new(config, organizations_repository),
      repository: RepositoryService::new(config, repositories_repository, git_backend.clone()),
      user,
      git_backend,
    }
  }
}
