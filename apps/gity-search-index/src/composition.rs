use fluxdi::{Shared, static_module};
use gity_core::{Db, Settings};
use gity_git::GitService;
use gity_search::{Limits, SearchIndex};
#[derive(Clone)]
pub struct Runtime {
  pub db: Db,
  pub settings: Shared<Settings>,
  pub search: SearchIndex,
}
fn git(settings: Shared<Settings>) -> Result<Shared<GitService>, fluxdi::Error> {
  Ok(Shared::new(GitService::new(
    &settings.repo_root,
    &settings.git_bin,
  )))
}
fn search(
  settings: Shared<Settings>,
  git: Shared<GitService>,
) -> Result<Shared<SearchIndex>, fluxdi::Error> {
  let limits = Limits {
    max_files: settings.search.max_files_per_project,
    max_file_size: settings.search.max_file_size_bytes,
    ..Default::default()
  };
  SearchIndex::new(&settings.search.index_root, (*git).clone(), limits)
    .map(Shared::new)
    .map_err(|e| fluxdi::Error::factory_failed("SearchIndex", &e.to_string()))
}
fn runtime(
  settings: Shared<Settings>,
  db: Shared<Db>,
  search: Shared<SearchIndex>,
) -> Result<Shared<Runtime>, fluxdi::Error> {
  Ok(Shared::new(Runtime {
    db: (*db).clone(),
    settings,
    search: (*search).clone(),
  }))
}
static_module! {
    pub struct SearchModule{
        #[scope(root)]runtime:Runtime=runtime(settings,database,search);
        #[scope(root)]search:SearchIndex=search(settings,git);
        #[scope(root)]git:GitService=git(settings);
        external settings:Settings;
        external database:Db;
    }
}
pub async fn compose(settings: Settings) -> anyhow::Result<(SearchModule, Runtime)> {
  let db = Db::connect(&settings.database_url).await?;
  let module = SearchModule::new(Shared::new(settings), Shared::new(db));
  let runtime = (*module.runtime()?).clone();
  Ok((module, runtime))
}
