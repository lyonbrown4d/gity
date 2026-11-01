use crate::AppState;
use fluxdi::{Shared, static_module};
use gity_core::{Db, Settings, Storage};
use gity_git::GitService;
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

fn git(settings: Shared<Settings>) -> Result<Shared<GitService>, fluxdi::Error> {
    Ok(Shared::new(GitService::new(
        &settings.repo_root,
        &settings.git_bin,
    )))
}
fn state(
    db: Shared<Db>,
    git: Shared<GitService>,
    settings: Shared<Settings>,
    storage: Shared<Storage>,
    search: Shared<SearchIndex>,
) -> Result<Shared<AppState>, fluxdi::Error> {
    Ok(Shared::new(AppState {
        db: (*db).clone(),
        git: (*git).clone(),
        settings,
        storage: (*storage).clone(),
        search: (*search).clone(),
    }))
}
// Missing dependencies, factory types, cycles and root scope are checked by FluxDI.
static_module! {
    pub struct RuntimeModule {
        #[scope(root)]
        state:AppState=state(database,git,settings,storage,search);
        #[scope(root)]
        git:GitService=git(settings);
        #[scope(root)]
        search:SearchIndex=search(settings,git);
        external settings:Settings;
        external database:Db;
        external storage:Storage;
    }
}
pub async fn compose(settings: Settings) -> anyhow::Result<(RuntimeModule, AppState)> {
    tokio::fs::create_dir_all(&settings.repo_root).await?;
    tokio::fs::create_dir_all(&settings.storage_root).await?;
    let db = Db::connect(&settings.database_url).await?;
    let storage = Storage::from_settings(&settings)?;
    let module = RuntimeModule::new(Shared::new(settings), Shared::new(db), Shared::new(storage));
    let state = (*module.state()?).clone();
    Ok((module, state))
}
