use crate::configuration::cfg::{CacheType, Config};
use crate::service::git_backend_service::GitBackendService;
use apalis::prelude::{TaskSink, WorkerBuilder};
use apalis_redis::{RedisConfig, RedisStorage};
use chrono::Utc;
use entity::{repository_language_snapshot_items, repository_language_snapshots};
use repository::{
  OrganizationsRepository, RepositoriesRepository, RepositoryBranchesRepository,
  RepositoryLanguageSnapshotsRepository,
};
use sea_orm::{DatabaseConnection, Set};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryLanguageJob {
  pub repository_id: String,
  pub branch_name: Option<String>,
}

#[derive(Clone)]
pub struct RepositoryLanguageJobClient {
  storage: RedisStorage<RepositoryLanguageJob>,
}

impl RepositoryLanguageJobClient {
  pub fn new(storage: RedisStorage<RepositoryLanguageJob>) -> Self {
    Self { storage }
  }

  pub async fn enqueue(&self, job: RepositoryLanguageJob) -> Result<(), String> {
    let mut queue = self.storage.clone();
    queue
      .push(job)
      .await
      .map_err(|err| format!("failed to enqueue repository language job: {err}"))
  }

  pub async fn enqueue_repository_branch(
    &self,
    repository_id: &str,
    branch_name: Option<&str>,
  ) -> Result<(), String> {
    self
      .enqueue(RepositoryLanguageJob {
        repository_id: repository_id.to_string(),
        branch_name: branch_name.map(str::to_string),
      })
      .await
  }
}

#[derive(Clone)]
struct RepositoryLanguageWorkerState {
  db_conn: DatabaseConnection,
  git_backend: GitBackendService,
}

pub async fn init_repository_language_jobs(
  config: &Config,
  db_conn: DatabaseConnection,
) -> Result<Option<RepositoryLanguageJobClient>, String> {
  let Some(cache) = config.cache.as_ref() else {
    return Ok(None);
  };
  if cache.cache_type != CacheType::REDIS {
    return Ok(None);
  }
  let cache_url = cache
    .url
    .as_deref()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .ok_or_else(|| "cache.url is required when cache.cache_type=REDIS".to_string())?;

  let conn = apalis_redis::connect(cache_url)
    .await
    .map_err(|err| format!("failed to connect apalis redis storage: {err}"))?;
  let queue = RedisStorage::new_with_config(
    conn,
    RedisConfig::new("gity.repository-language").set_buffer_size(100),
  );
  let client = RepositoryLanguageJobClient::new(queue.clone());

  let worker_state = RepositoryLanguageWorkerState {
    db_conn: db_conn.clone(),
    git_backend: GitBackendService::new(config, RepositoryBranchesRepository::new(db_conn)),
  };
  tokio::spawn(async move {
    let state = worker_state.clone();
    let worker = WorkerBuilder::new("repository-language-worker")
      .backend(queue)
      .build(move |job: RepositoryLanguageJob| {
        let state = state.clone();
        async move {
          process_repository_language_job(job, state).await;
          Ok::<(), apalis::prelude::BoxDynError>(())
        }
      });

    if let Err(err) = worker.run().await {
      warn!(
        error = err.to_string(),
        "repository language worker stopped unexpectedly"
      );
    }
  });

  info!("repository language worker initialized");
  Ok(Some(client))
}

async fn process_repository_language_job(
  job: RepositoryLanguageJob,
  state: RepositoryLanguageWorkerState,
) {
  let repository = match RepositoriesRepository::new(state.db_conn.clone())
    .find_active_repository_by_id(job.repository_id.as_str())
    .await
  {
    Ok(Some(repository)) => repository,
    Ok(None) => return,
    Err(err) => {
      warn!(
        repository_id = job.repository_id,
        error = err.to_string(),
        "failed to load repository for language job"
      );
      return;
    }
  };

  let organization = match OrganizationsRepository::new(state.db_conn.clone())
    .find_active_organization_by_id(repository.organization_id.as_str())
    .await
  {
    Ok(Some(organization)) => organization,
    Ok(None) => return,
    Err(err) => {
      warn!(
        repository_id = repository.id,
        error = err.to_string(),
        "failed to load organization for language job"
      );
      return;
    }
  };

  let branch_name = job
    .branch_name
    .unwrap_or_else(|| repository.default_branch.clone());
  let branch = match RepositoryBranchesRepository::new(state.db_conn.clone())
    .find_active_branch_by_repo_and_name(repository.id.as_str(), branch_name.as_str())
    .await
  {
    Ok(Some(branch)) => branch,
    Ok(None) => return,
    Err(err) => {
      warn!(
        repository_id = repository.id,
        branch = branch_name,
        error = err.to_string(),
        "failed to load repository branch for language job"
      );
      return;
    }
  };

  let revision = branch
    .last_commit_sha
    .clone()
    .unwrap_or_else(|| format!("refs/heads/{}:empty", branch_name));

  let language_stats = if let Some(commit_sha) = branch.last_commit_sha.clone() {
    match state
      .git_backend
      .analyze_languages(
        organization.key.as_str(),
        repository.key.as_str(),
        commit_sha.as_str(),
      )
      .await
    {
      Ok(stats) => stats,
      Err(err) => {
        warn!(
          organization = organization.key,
          repository = repository.key,
          branch = branch_name,
          error = err.to_string(),
          "failed to analyze repository languages"
        );
        return;
      }
    }
  } else {
    Vec::new()
  };

  if let Err(err) = persist_snapshot(
    &state.db_conn,
    repository.id.as_str(),
    branch_name.as_str(),
    revision.as_str(),
    &language_stats,
  )
  .await
  {
    warn!(
      repository_id = repository.id,
      branch = branch_name,
      error = err.to_string(),
      "failed to persist repository language snapshot"
    );
  }
}

async fn persist_snapshot(
  db_conn: &DatabaseConnection,
  repository_id: &str,
  branch_name: &str,
  revision: &str,
  stats: &[git::object::RepositoryLanguageStat],
) -> Result<(), String> {
  let snapshots_repository = RepositoryLanguageSnapshotsRepository::new(db_conn.clone());
  let analyzed_at = Utc::now().into();
  let total_bytes = stats.iter().fold(0_i64, |acc, stat| {
    acc.saturating_add(saturating_u64_to_i64(stat.bytes))
  });

  let snapshot = snapshots_repository
    .insert_snapshot(repository_language_snapshots::ActiveModel {
      repository_id: Set(repository_id.to_string()),
      branch_name: Set(branch_name.to_string()),
      revision: Set(revision.to_string()),
      total_bytes: Set(total_bytes),
      analyzed_at: Set(analyzed_at),
      created_at: Set(analyzed_at),
      ..Default::default()
    })
    .await
    .map_err(|err| format!("failed to insert language snapshot: {err}"))?;

  for stat in stats {
    snapshots_repository
      .insert_snapshot_item(repository_language_snapshot_items::ActiveModel {
        snapshot_id: Set(snapshot.id.clone()),
        language: Set(stat.language.clone()),
        bytes: Set(saturating_u64_to_i64(stat.bytes)),
        created_at: Set(analyzed_at),
        ..Default::default()
      })
      .await
      .map_err(|err| format!("failed to insert language snapshot item: {err}"))?;
  }

  Ok(())
}

fn saturating_u64_to_i64(value: u64) -> i64 {
  value.min(i64::MAX as u64) as i64
}
