use crate::AppState;
use fluxdi::{Shared, static_module};
#[cfg(feature = "packages")]
use gity_artifacts::{ArtifactModule, ArtifactService};
pub struct ArtifactBindings {
  #[cfg(feature = "packages")]
  service: Shared<ArtifactService>,
}
use gity_core::{Db, Settings, Storage};
use gity_git::GitService;
use gity_identity::{IdentityModule, IdentityRepository};
use gity_search::{Limits, SearchIndex};
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
  identity: Shared<IdentityRepository>,
  _artifacts: Shared<ArtifactBindings>,
) -> Result<Shared<AppState>, fluxdi::Error> {
  Ok(Shared::new(AppState {
    db: (*db).clone(),
    git: backend.git.clone(),
    git_cluster: backend.cluster.clone(),
    settings,
    storage: (*storage).clone(),
    search: (*search).clone(),
    identity,
    #[cfg(feature = "packages")]
    artifacts: _artifacts.service.clone(),
  }))
}
// Missing dependencies, factory types, cycles and root scope are checked by FluxDI.
static_module! {
    pub struct RuntimeModule {
        import identity_module: IdentityModule {
            repository: IdentityRepository;
        };
        #[scope(root)]
        state:AppState=state(database,backend,settings,storage,search,repository,artifacts);
        #[scope(root)]
        git:GitService=git(backend);
        #[scope(root)]
        backend:Backend=backend(settings,database);
        #[scope(root)]
        search:SearchIndex=search(settings,git);
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
    service: ArtifactModule::new(connection, storage.clone()).service()?,
  });
  #[cfg(not(feature = "packages"))]
  let artifacts = Shared::new(ArtifactBindings {});
  let module = RuntimeModule::new(
    identity,
    Shared::new(settings),
    Shared::new(db),
    storage,
    artifacts,
  );
  let state = (*module.state()?).clone();
  Ok((module, state))
}
