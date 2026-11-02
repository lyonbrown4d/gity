use crate::{
  ProjectRepository, ProjectService, RepositoryAccessRepository, RepositoryAccessService,
};
use fluxdi::module::Module;
use fluxdi::{Injector, Provider, Shared, static_module};
use sea_orm::DatabaseConnection;
fn project_repository(
  connection: Shared<DatabaseConnection>,
) -> Result<Shared<ProjectRepository>, fluxdi::Error> {
  Ok(Shared::new(ProjectRepository::new((*connection).clone())))
}
fn access_repository(
  connection: Shared<DatabaseConnection>,
) -> Result<Shared<RepositoryAccessRepository>, fluxdi::Error> {
  Ok(Shared::new(RepositoryAccessRepository::new(
    (*connection).clone(),
  )))
}
fn projects(
  repository: Shared<ProjectRepository>,
) -> Result<Shared<ProjectService>, fluxdi::Error> {
  Ok(Shared::new(ProjectService::new((*repository).clone())))
}
fn access(
  repository: Shared<RepositoryAccessRepository>,
) -> Result<Shared<RepositoryAccessService>, fluxdi::Error> {
  Ok(Shared::new(RepositoryAccessService::new(
    (*repository).clone(),
  )))
}
static_module! {
  pub struct ProjectModule {
    #[scope(root)]
    projects_repository: ProjectRepository = project_repository(connection);
    #[scope(root)]
    access_repository: RepositoryAccessRepository = access_repository(connection);
    #[scope(root)]
    service: ProjectService = projects(projects_repository);
    #[scope(root)]
    repository_access: RepositoryAccessService = access(access_repository);
    external connection: DatabaseConnection;
  }
}
impl Module for ProjectModule {
  fn configure(&self, injector: &Injector) -> Result<(), fluxdi::Error> {
    let service = self.service()?;
    injector.provide(Provider::<ProjectService>::root(move |_| service.clone()));
    let access = self.repository_access()?;
    injector.provide(Provider::<RepositoryAccessService>::root(move |_| {
      access.clone()
    }));
    Ok(())
  }
}
