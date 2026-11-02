use crate::AppState;
use fluxdi::{Shared, static_module};
#[cfg(feature = "packages")]
use gity_artifacts::{ArtifactModule, ArtifactService};
pub struct ArtifactBindings {
  #[cfg(feature = "packages")]
  service: Shared<ArtifactService>,
}
use gity_ci::{CiModule, CiService};
use gity_collaboration::{CollaborationModule, CollaborationService};
use gity_core::{Db, IdGenerator, Settings, Storage, install};
use gity_git::GitService;
use gity_identity::{IdentityModule, IdentityRepository, OAuthModule, OAuthService};
use gity_projects::{ProjectModule, ProjectService, RepositoryAccessService};

struct ApplicationServices {
  identity: Shared<IdentityRepository>,
  oauth: Shared<OAuthService>,
  collaboration: Shared<CollaborationService>,
  ci: Shared<CiService>,
  projects: Shared<ProjectService>,
  repository_access: Shared<RepositoryAccessService>,
}
fn services(
  identity: Shared<IdentityRepository>,
  oauth: Shared<OAuthService>,
  collaboration: Shared<CollaborationService>,
  ci: Shared<CiService>,
  projects: Shared<ProjectService>,
  repository_access: Shared<RepositoryAccessService>,
) -> Result<Shared<ApplicationServices>, fluxdi::Error> {
  Ok(Shared::new(ApplicationServices {
    identity,
    oauth,
    collaboration,
    ci,
    projects,
    repository_access,
  }))
}
use gity_search::{Limits, SearchIndex};
fn id_generator(settings: Shared<Settings>) -> Result<Shared<IdGenerator>, fluxdi::Error> {
  IdGenerator::new(settings.database_node_id)
    .map(Shared::new)
    .map_err(|error| fluxdi::Error::factory_failed("IdGenerator", &error.to_string()))
}
fn search(
  settings: Shared<Settings>,
  git: Shared<GitService>,
) -> Result<Shared<SearchIndex>, fluxdi::Error> {
  SearchIndex::new(
    &settings.search.index_root,
    (*git).clone(),
    Limits {
      max_files: settings.search.max_files_per_project,
      max_file_size: settings.search.max_file_size_bytes,
      ..Limits::default()
    },
  )
  .map(Shared::new)
  .map_err(|e| fluxdi::Error::factory_failed("SearchIndex", &e.to_string()))
}

struct Backend {
  git: GitService,
  cluster: Option<gity_git_cluster::GitClusterModule>,
}
fn backend(
  settings: Shared<Settings>,
  database: Shared<Db>,
) -> Result<Shared<Backend>, fluxdi::Error> {
  let cluster = settings
    .git_cluster
    .as_ref()
    .map(|config| {
      gity_git_cluster::GitClusterModule::new(config.clone(), database.connection.clone())
    })
    .transpose()?;
  let git = if let Some(module) = &cluster {
    GitService::with_remote(module.catalog()?)
  } else {
    GitService::new(&settings.repo_root, &settings.git_bin)
  };
  Ok(Shared::new(Backend { git, cluster }))
}
fn git(backend: Shared<Backend>) -> Result<Shared<GitService>, fluxdi::Error> {
  Ok(Shared::new(backend.git.clone()))
}
fn state(
  db: Shared<Db>,
  backend: Shared<Backend>,
  settings: Shared<Settings>,
  storage: Shared<Storage>,
  search: Shared<SearchIndex>,
  services: Shared<ApplicationServices>,
  _artifacts: Shared<ArtifactBindings>,
) -> Result<Shared<AppState>, fluxdi::Error> {
  Ok(Shared::new(AppState {
    db: (*db).clone(),
    git: backend.git.clone(),
    git_cluster: backend.cluster.clone(),
    settings,
    storage: (*storage).clone(),
    search: (*search).clone(),
    identity: services.identity.clone(),
    oauth: services.oauth.clone(),
    collaboration: services.collaboration.clone(),
    ci: services.ci.clone(),
    projects: services.projects.clone(),
    repository_access: services.repository_access.clone(),
    #[cfg(feature = "packages")]
    artifacts: _artifacts.service.clone(),
  }))
}
// Missing dependencies, factory types, cycles and root scope are checked by FluxDI.
static_module! {
    pub struct ServicesModule {
        import oauth_module: OAuthModule { oauth: OAuthService = service; };
        import identity_module: IdentityModule {
            repository: IdentityRepository;
        };
        import collaboration_module: CollaborationModule {
            collaboration: CollaborationService = service;
        };
        import ci_module: CiModule {
            ci: CiService = service;
        };
        import projects_module: ProjectModule {
            projects: ProjectService = service;
            repository_access: RepositoryAccessService;
        };
        #[scope(root)]
        services: ApplicationServices=services(repository,oauth,collaboration,ci,projects,repository_access);
    }
}
static_module! {
    pub struct RuntimeModule {
        import services_module: ServicesModule { services: ApplicationServices; };
        #[scope(root)]
        state:AppState=state(database,backend,settings,storage,search,services,artifacts);
        #[scope(root)]
        git:GitService=git(backend);
        #[scope(root)]
        backend:Backend=backend(settings,database);
        #[scope(root)]
        search:SearchIndex=search(settings,git);
        #[scope(root)]
        id_generator:IdGenerator=id_generator(settings);
        external settings:Settings;
        external database:Db;
        external storage:Storage;
        external artifacts:ArtifactBindings;
    }
}
pub async fn compose(settings: Settings) -> anyhow::Result<(RuntimeModule, AppState)> {
  if settings.git_cluster.is_none() {
    tokio::fs::create_dir_all(&settings.repo_root).await?;
  }
  tokio::fs::create_dir_all(&settings.storage_root).await?;
  let db = Db::connect(&settings.database_url).await?;
  let storage = Storage::from_settings(&settings)?;
  let connection = Shared::new(db.connection.clone());
  let storage = Shared::new(storage);
  let identity = IdentityModule::new(connection.clone());
  #[cfg(feature = "packages")]
  let artifacts = Shared::new(ArtifactBindings {
    service: ArtifactModule::new(connection.clone(), storage.clone()).service()?,
  });
  #[cfg(not(feature = "packages"))]
  let artifacts = Shared::new(ArtifactBindings {});
  let services = ServicesModule::new(
    OAuthModule::new(
      Shared::new(db.connection.clone()),
      Shared::new(settings.oauth.clone()),
      Shared::new(settings.base_url.clone()),
    ),
    identity,
    CollaborationModule::new(connection.clone()),
    CiModule::new(connection.clone()),
    ProjectModule::new(connection.clone()),
  );
  let module = RuntimeModule::new(
    services,
    Shared::new(settings),
    Shared::new(db),
    storage,
    artifacts,
  );
  // Install the container's own generator first so entity hooks and the injector share one sequence.
  install((*module.id_generator()?).clone())?;
  let state = (*module.state()?).clone();
  Ok((module, state))
}
